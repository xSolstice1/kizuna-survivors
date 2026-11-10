// Cloud saves. Fill in your Supabase project's URL and anon (public) key to turn them on;
// leave them empty and the game keeps saving to this browser only. The anon key is safe to
// ship: row-level security only lets a signed-in player read their own rows, and every write
// goes through the server functions in supabase/migrations.
window.KZ_CLOUD = {
  url: "",
  anonKey: "",
  // Sign-in buttons to show (each must be enabled in Supabase → Authentication → Providers).
  providers: ["google", "discord", "email"],
};
