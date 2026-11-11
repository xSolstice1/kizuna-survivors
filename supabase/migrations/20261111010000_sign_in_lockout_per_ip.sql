-- Username sign-in lockout per caller, not per username.
--
-- 20261110020000 locked a username after 10 wrong passwords from anyone, so a stranger could
-- keep any player out of username sign-in. Now the 10-in-15-minutes limit counts per username
-- and caller IP; a ceiling of 100 failures per username across all IPs still bounds password
-- guessing from many addresses. Email sign-in is never locked by this.
--
-- Keeps every existing row (they become ip ''); expired rows are pruned on each call.
-- Same RPC signature, so no client change. Safe to re-run.
--
-- Rollback: re-run 20261110020000_username_sign_in.sql's create function, then
--   delete from public.sign_in_attempts where ip <> '';
--   alter table public.sign_in_attempts drop constraint sign_in_attempts_pkey,
--     add primary key (username), drop column ip;

alter table public.sign_in_attempts add column if not exists ip text not null default '';
alter table public.sign_in_attempts drop constraint if exists sign_in_attempts_pkey;
alter table public.sign_in_attempts add primary key (username, ip);
create index if not exists sign_in_attempts_window_start on public.sign_in_attempts (window_start);

create or replace function public.email_for_sign_in(p_username text, p_password text)
returns text
language plpgsql security definer set search_path = public, extensions
as $$
declare
  k text := lower(p_username);
  hdr json := nullif(current_setting('request.headers', true), '')::json;
  -- Addresses set by Supabase's edge, never the caller's own: Cloudflare's client IP, else the
  -- last X-Forwarded-For hop (a caller can only prepend entries, not append).
  caller text := coalesce(nullif(trim(hdr->>'cf-connecting-ip'), ''), nullif(trim(hdr->>'x-real-ip'), ''),
    nullif(trim(regexp_replace(coalesce(hdr->>'x-forwarded-for', ''), '^.*,', '')), ''), 'unknown');
  mine int;
  total int;
  uid uuid;
  mail text;
  hash text;
begin
  if p_username is null or p_password is null or not (p_username ~ '^[A-Za-z0-9_]{3,16}$') then
    return null;
  end if;

  delete from public.sign_in_attempts where window_start <= now() - interval '15 minutes';
  select coalesce(sum(failures) filter (where s.ip = caller), 0), coalesce(sum(failures), 0)
    into mine, total
    from public.sign_in_attempts s where s.username = k;
  if mine >= 10 or total >= 100 then
    raise exception 'too many attempts' using errcode = 'P0429';
  end if;

  -- The profile's username, or (before the first save) the one chosen at sign-up.
  select p.user_id into uid from public.profiles p where lower(p.username) = k;
  if uid is null then
    select u.id into uid from auth.users u where lower(u.raw_user_meta_data->>'username') = k
    order by u.created_at limit 1;
  end if;
  if uid is not null then
    select u.email, u.encrypted_password into mail, hash from auth.users u where u.id = uid;
  end if;

  if hash is not null and hash <> '' and hash = crypt(p_password, hash) then
    delete from public.sign_in_attempts where username = k;
    return mail;
  end if;

  insert into public.sign_in_attempts as s (username, ip, failures, window_start) values (k, caller, 1, now())
  on conflict (username, ip) do update set failures = s.failures + 1;
  return null;
end $$;

revoke all on function public.email_for_sign_in(text, text) from public;
grant execute on function public.email_for_sign_in(text, text) to anon, authenticated;
