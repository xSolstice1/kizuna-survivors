# Cloud saves (Supabase)

Accounts and database-stored saves. Off until configured: with an empty `web/config.js` the
game keeps saving to the browser only.

## How it fits together

- **Database** (`supabase/migrations/20261110000000_cloud_saves.sql`)
  - `profiles`: one row per player: `sen` and `dust` as real columns (never negative) plus
    `doc` (jsonb) for the rest of the save, and a `version` for optimistic concurrency.
  - `items`: one row per piece of gear, with queryable columns (slot, rarity, ilvl, plus, set,
    exotic, unique, element, locked, affixes) and `data`, the game's own item encoding.
  - `loadouts`: which hero wears which item in which slot (an item can be worn once).
  - `runs`: one row per finished night (hero, stage, difficulty, result, time, kills, gear).
  - `currency_ledger`: every Sen / Spirit Dust change with its reason.
  - Row-level security: players can **read** only their own rows; there are **no** write
    policies. All writes go through `SECURITY DEFINER` functions: `get_state()`,
    `sync_state(...)` (one transaction: profile, item upserts and deletions, loadout) and
    `record_run(...)`. They trust the client today; add validation inside them later
    (server-rolled loot, run replay, trades) without changing the client or the data.
- **Browser** (`web/cloud.js`): sign-in through Supabase Auth's REST API (Google, Discord or an
  email link), and local-first sync: the game hands over its state on every save, the plugin
  sends only the difference (debounced), keeps a queue of runs while offline, and on sign-in
  reconciles: upload a new account, take the cloud copy if only the cloud changed, or ask the
  player (Settings → Account) when both changed.
- **Game** (`crates/client/src/cloud.rs`): converts the save to and from the database shape;
  Settings → Account shows sign-in buttons, sync status, sign-out and the conflict choice.

## Setting it up

1. Create a Supabase project.
2. Run the migration: Supabase dashboard → SQL Editor → paste the file and run it (or
   `supabase db push` with the Supabase CLI linked to the project).
3. Authentication → URL Configuration: set **Site URL** to `https://kizuna-survivors.vercel.app`
   and add it (plus `http://localhost:8000` for local testing) to **Redirect URLs**.
4. Authentication → Providers: enable the ones you want.
   - Google: create an OAuth client in Google Cloud Console; authorised redirect URI is
     `https://<project>.supabase.co/auth/v1/callback`.
   - Discord: create an application at discord.com/developers; same redirect URI.
   - Email: on by default (magic links). Set up custom SMTP for real traffic.
5. Put the project URL and the **anon** key (Project Settings → API) in `web/config.js`, and
   trim `providers` to the ones you enabled. Commit and push; Vercel redeploys.

Free-tier projects pause after a week without traffic.
