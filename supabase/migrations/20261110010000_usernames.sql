-- Usernames: a unique in-game name per player (3–16 letters, digits or underscores),
-- case-insensitive. Set from the sign-up form (auth user metadata `username`) when the
-- profile row is first created; Google players can pick one later with set_username().

alter table public.profiles add column if not exists username text;
alter table public.profiles drop constraint if exists profiles_username_format;
alter table public.profiles add constraint profiles_username_format check (username is null or username ~ '^[A-Za-z0-9_]{3,16}$');
create unique index if not exists profiles_username_key on public.profiles (lower(username));

-- Whether a username is free (callable before signing up; reveals nothing else).
create or replace function public.username_available(p_username text)
returns boolean
language sql security definer set search_path = public stable
as $$
  select p_username ~ '^[A-Za-z0-9_]{3,16}$'
     and not exists (select 1 from public.profiles where lower(username) = lower(p_username))
     and not exists (select 1 from auth.users where lower(raw_user_meta_data->>'username') = lower(p_username));
$$;

-- Picks (or changes) the signed-in player's username.
create or replace function public.set_username(p_username text)
returns void
language plpgsql security definer set search_path = public
as $$
begin
  if auth.uid() is null then raise exception 'not signed in'; end if;
  if not (p_username ~ '^[A-Za-z0-9_]{3,16}$') then raise exception 'invalid username' using errcode = 'P0422'; end if;
  insert into public.profiles (user_id, username) values (auth.uid(), p_username)
  on conflict (user_id) do update set username = excluded.username, updated_at = now();
exception when unique_violation then
  raise exception 'username taken' using errcode = 'P0423';
end $$;

-- New profiles take the username chosen at sign-up.
create or replace function public.profile_username_from_signup()
returns trigger
language plpgsql security definer set search_path = public
as $$
begin
  if new.username is null then
    select nullif(raw_user_meta_data->>'username', '') into new.username from auth.users where id = new.user_id;
    if new.username is not null and not (new.username ~ '^[A-Za-z0-9_]{3,16}$') then new.username := null; end if;
  end if;
  return new;
end $$;
drop trigger if exists profiles_username_from_signup on public.profiles;
create trigger profiles_username_from_signup before insert on public.profiles
  for each row execute function public.profile_username_from_signup();

-- get_state() also returns the username.
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
    'exists', p.version > 0,
    'username', p.username,
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

revoke all on function public.username_available(text) from public;
grant execute on function public.username_available(text) to anon, authenticated;
revoke all on function public.set_username(text) from public, anon;
grant execute on function public.set_username(text) to authenticated;
