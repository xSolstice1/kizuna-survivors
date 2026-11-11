-- Sign in with a username instead of an email.
--
-- email_for_sign_in(username, password) returns the account's email only when the password
-- is right (checked against the bcrypt hash Supabase Auth keeps), so nobody can look up an
-- email from a username. The game then signs in with that email through Supabase Auth as
-- usual. Wrong guesses are counted per username: 10 within 15 minutes lock it for the rest
-- of the window.

create extension if not exists pgcrypto with schema extensions;

create table if not exists public.sign_in_attempts (
  username text primary key,
  failures int not null default 0,
  window_start timestamptz not null default now()
);
alter table public.sign_in_attempts enable row level security;  -- no policies: functions only
revoke all on public.sign_in_attempts from anon, authenticated;

create or replace function public.email_for_sign_in(p_username text, p_password text)
returns text
language plpgsql security definer set search_path = public, extensions
as $$
declare
  k text := lower(p_username);
  a public.sign_in_attempts;
  uid uuid;
  mail text;
  hash text;
begin
  if p_username is null or p_password is null or not (p_username ~ '^[A-Za-z0-9_]{3,16}$') then
    return null;
  end if;

  select * into a from public.sign_in_attempts where username = k;
  if found and a.window_start > now() - interval '15 minutes' and a.failures >= 10 then
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

  insert into public.sign_in_attempts as s (username, failures, window_start) values (k, 1, now())
  on conflict (username) do update set
    failures = case when s.window_start > now() - interval '15 minutes' then s.failures + 1 else 1 end,
    window_start = case when s.window_start > now() - interval '15 minutes' then s.window_start else now() end;
  return null;
end $$;

revoke all on function public.email_for_sign_in(text, text) from public;
grant execute on function public.email_for_sign_in(text, text) to anon, authenticated;
