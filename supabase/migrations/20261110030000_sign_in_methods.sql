-- How an email signs in, so the game can point a Google player at "Continue with Google"
-- instead of failing a password sign-in or sign-up with that email.
--
-- Returns e.g. {google}, {email,password} or {} (no account). This tells a caller whether an
-- email has an account and which method it uses; Supabase's sign-up reply already shows
-- whether an email is registered, so the method is the only new detail.

create or replace function public.sign_in_methods(p_email text)
returns text[]
language sql security definer set search_path = public stable
as $$
  select coalesce(array_agg(m order by m), '{}')
  from (
    select distinct i.provider as m
    from auth.users u join auth.identities i on i.user_id = u.id
    where lower(u.email) = lower(trim(p_email))
    union
    select 'password'
    from auth.users u
    where lower(u.email) = lower(trim(p_email)) and coalesce(u.encrypted_password, '') <> ''
  ) s;
$$;

revoke all on function public.sign_in_methods(text) from public;
grant execute on function public.sign_in_methods(text) to anon, authenticated;
