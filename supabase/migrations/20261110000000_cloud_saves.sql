-- Kizuna Survivors cloud saves.
--
-- Shape: a profile row per player (currencies as real columns, the rest of the save as a
-- jsonb document), one row per item of gear, one row per equipped slot, one row per finished
-- run, and an append-only currency ledger.
--
-- Security: every table has row-level security. Players may READ their own rows; there are
-- NO insert/update/delete policies, so all writes go through the SECURITY DEFINER functions
-- below. Today those functions trust what the client sends (the game is single-player and the
-- simulation runs in the browser); validation (replaying runs, server-rolled loot, trades) can
-- be added inside them later without changing the client or moving data.

create table if not exists public.profiles (
  user_id     uuid primary key references auth.users (id) on delete cascade,
  sen         bigint not null default 0 check (sen >= 0),
  dust        bigint not null default 0 check (dust >= 0),
  doc         jsonb  not null default '{}'::jsonb,   -- everything else in the save (key -> value)
  version     bigint not null default 0,             -- bumped on every sync
  created_at  timestamptz not null default now(),
  updated_at  timestamptz not null default now()
);

create table if not exists public.items (
  user_id     uuid   not null references auth.users (id) on delete cascade,
  item_id     bigint not null,                       -- the game's item id (unique per player)
  slot        text   not null,
  rarity      text   not null,
  ilvl        int    not null check (ilvl >= 1),
  plus        int    not null default 0 check (plus between 0 and 10),
  set_key     text,
  exotic      text,
  unique_key  text,
  element     text,
  locked      boolean not null default false,
  affixes     jsonb  not null default '[]'::jsonb,   -- [{stat, quality}]
  data        text   not null,                       -- the game's full item encoding (source of truth)
  created_at  timestamptz not null default now(),
  updated_at  timestamptz not null default now(),
  primary key (user_id, item_id)
);
create index if not exists items_rarity_idx on public.items (rarity);
create index if not exists items_exotic_idx on public.items (exotic) where exotic is not null;

create table if not exists public.loadouts (
  user_id     uuid   not null references auth.users (id) on delete cascade,
  hero        text   not null,
  pos         int    not null check (pos between 0 and 11),
  item_id     bigint not null,
  primary key (user_id, hero, pos),
  unique (user_id, item_id),                         -- an item is worn by at most one hero, once
  foreign key (user_id, item_id) references public.items (user_id, item_id) on delete cascade
);

create table if not exists public.runs (
  id          bigint generated always as identity primary key,
  user_id     uuid   not null references auth.users (id) on delete cascade,
  hero        text   not null,
  stage       text   not null,
  difficulty  text   not null,
  ascension   int    not null default 0,
  result      text   not null check (result in ('won', 'fallen', 'dawn')),
  seconds     real   not null,
  level       int    not null,
  kills       bigint not null,
  sen         bigint not null default 0,
  gear_found  int    not null default 0,
  detail      jsonb  not null default '{}'::jsonb,   -- weapons, awakenings, bosses, ...
  played_at   timestamptz not null default now()
);
create index if not exists runs_user_idx on public.runs (user_id, played_at desc);
create index if not exists runs_board_idx on public.runs (stage, difficulty, result, seconds);

create table if not exists public.currency_ledger (
  id          bigint generated always as identity primary key,
  user_id     uuid   not null references auth.users (id) on delete cascade,
  currency    text   not null check (currency in ('sen', 'dust')),
  delta       bigint not null,
  balance     bigint not null,
  reason      text   not null,
  at          timestamptz not null default now()
);
create index if not exists ledger_user_idx on public.currency_ledger (user_id, at desc);

alter table public.profiles        enable row level security;
alter table public.items           enable row level security;
alter table public.loadouts        enable row level security;
alter table public.runs            enable row level security;
alter table public.currency_ledger enable row level security;

drop policy if exists "read own profile" on public.profiles;
create policy "read own profile" on public.profiles for select using (auth.uid() = user_id);
drop policy if exists "read own items" on public.items;
create policy "read own items" on public.items for select using (auth.uid() = user_id);
drop policy if exists "read own loadouts" on public.loadouts;
create policy "read own loadouts" on public.loadouts for select using (auth.uid() = user_id);
drop policy if exists "read own runs" on public.runs;
create policy "read own runs" on public.runs for select using (auth.uid() = user_id);
drop policy if exists "read own ledger" on public.currency_ledger;
create policy "read own ledger" on public.currency_ledger for select using (auth.uid() = user_id);

-- ───────────── Server functions (the only write path) ─────────────

-- The player's whole cloud state, or {"exists": false} for a new account.
create or replace function public.get_state()
returns jsonb
language plpgsql security definer set search_path = public
as $$
declare
  uid uuid := auth.uid();
  p public.profiles;
begin
  if uid is null then raise exception 'not signed in'; end if;
  select * into p from public.profiles where user_id = uid;
  if not found then return jsonb_build_object('exists', false); end if;
  return jsonb_build_object(
    'exists', true,
    'version', p.version,
    'updated_at', p.updated_at,
    'profile', jsonb_build_object('sen', p.sen, 'dust', p.dust, 'doc', p.doc),
    'items', coalesce((select jsonb_agg(jsonb_build_object(
        'item_id', i.item_id, 'slot', i.slot, 'rarity', i.rarity, 'ilvl', i.ilvl, 'plus', i.plus,
        'set', i.set_key, 'exotic', i.exotic, 'unique', i.unique_key, 'element', i.element,
        'locked', i.locked, 'affixes', i.affixes, 'data', i.data) order by i.item_id)
      from public.items i where i.user_id = uid), '[]'::jsonb),
    'loadouts', coalesce((select jsonb_agg(jsonb_build_object('hero', l.hero, 'pos', l.pos, 'item_id', l.item_id) order by l.hero, l.pos)
      from public.loadouts l where l.user_id = uid), '[]'::jsonb)
  );
end $$;

-- Applies one sync from the client in a single transaction: the profile row, item upserts and
-- deletions, and the full loadout. `p_base_version` must match the server's version (optimistic
-- concurrency: a stale device is told to pull first). Returns the new version.
create or replace function public.sync_state(
  p_base_version bigint,
  p_profile jsonb,
  p_upsert jsonb,       -- array of items, same shape as get_state()
  p_delete bigint[],    -- item ids to remove
  p_loadouts jsonb,     -- full loadout: array of {hero, pos, item_id}
  p_reason text default 'sync'
)
returns bigint
language plpgsql security definer set search_path = public
as $$
declare
  uid uuid := auth.uid();
  cur public.profiles;
  new_sen bigint := coalesce((p_profile->>'sen')::bigint, 0);
  new_dust bigint := coalesce((p_profile->>'dust')::bigint, 0);
begin
  if uid is null then raise exception 'not signed in'; end if;
  insert into public.profiles (user_id) values (uid) on conflict (user_id) do nothing;
  select * into cur from public.profiles where user_id = uid for update;
  if cur.version <> p_base_version then
    raise exception 'stale version % (server is %)', p_base_version, cur.version using errcode = 'P0409';
  end if;
  if jsonb_array_length(coalesce(p_upsert, '[]'::jsonb)) > 500 then raise exception 'too many items'; end if;

  -- Currencies, with a ledger entry for every change.
  if new_sen <> cur.sen then
    insert into public.currency_ledger (user_id, currency, delta, balance, reason) values (uid, 'sen', new_sen - cur.sen, new_sen, p_reason);
  end if;
  if new_dust <> cur.dust then
    insert into public.currency_ledger (user_id, currency, delta, balance, reason) values (uid, 'dust', new_dust - cur.dust, new_dust, p_reason);
  end if;
  update public.profiles
     set sen = new_sen, dust = new_dust, doc = coalesce(p_profile->'doc', doc),
         version = cur.version + 1, updated_at = now()
   where user_id = uid;

  -- Loadouts reference items, so clear them before touching items.
  delete from public.loadouts where user_id = uid;
  if p_delete is not null and array_length(p_delete, 1) > 0 then
    delete from public.items where user_id = uid and item_id = any (p_delete);
  end if;
  insert into public.items as t (user_id, item_id, slot, rarity, ilvl, plus, set_key, exotic, unique_key, element, locked, affixes, data)
  select uid, (x->>'item_id')::bigint, x->>'slot', x->>'rarity', (x->>'ilvl')::int, (x->>'plus')::int,
         x->>'set', x->>'exotic', x->>'unique', x->>'element', coalesce((x->>'locked')::boolean, false),
         coalesce(x->'affixes', '[]'::jsonb), x->>'data'
    from jsonb_array_elements(coalesce(p_upsert, '[]'::jsonb)) x
  on conflict (user_id, item_id) do update
     set slot = excluded.slot, rarity = excluded.rarity, ilvl = excluded.ilvl, plus = excluded.plus,
         set_key = excluded.set_key, exotic = excluded.exotic, unique_key = excluded.unique_key,
         element = excluded.element, locked = excluded.locked, affixes = excluded.affixes,
         data = excluded.data, updated_at = now();
  if (select count(*) from public.items where user_id = uid) > 1000 then raise exception 'bag too large'; end if;

  insert into public.loadouts (user_id, hero, pos, item_id)
  select uid, x->>'hero', (x->>'pos')::int, (x->>'item_id')::bigint
    from jsonb_array_elements(coalesce(p_loadouts, '[]'::jsonb)) x;

  return cur.version + 1;
end $$;

-- Logs one finished night.
create or replace function public.record_run(p_run jsonb)
returns void
language plpgsql security definer set search_path = public
as $$
begin
  if auth.uid() is null then raise exception 'not signed in'; end if;
  insert into public.runs (user_id, hero, stage, difficulty, ascension, result, seconds, level, kills, sen, gear_found, detail)
  values (auth.uid(), p_run->>'hero', p_run->>'stage', p_run->>'difficulty', coalesce((p_run->>'ascension')::int, 0),
          p_run->>'result', (p_run->>'seconds')::real, (p_run->>'level')::int, (p_run->>'kills')::bigint,
          coalesce((p_run->>'sen')::bigint, 0), coalesce((p_run->>'gear_found')::int, 0), coalesce(p_run->'detail', '{}'::jsonb));
end $$;

revoke all on function public.get_state() from public, anon;
revoke all on function public.sync_state(bigint, jsonb, jsonb, bigint[], jsonb, text) from public, anon;
revoke all on function public.record_run(jsonb) from public, anon;
grant execute on function public.get_state() to authenticated;
grant execute on function public.sync_state(bigint, jsonb, jsonb, bigint[], jsonb, text) to authenticated;
grant execute on function public.record_run(jsonb) to authenticated;
