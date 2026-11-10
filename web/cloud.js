// Cloud saves (Supabase): sign-in and sync for crates/client/src/cloud.rs.
//
// The game stays local-first. localStorage keeps working offline; this plugin mirrors the save
// to the database through the server functions in supabase/migrations (get_state, sync_state,
// record_run). Configure it in config.js; with no URL it stays off and the game is unchanged.
//
// Sync model:
//   - The game hands us its whole state as JSON whenever it saves (kz_cloud_push).
//   - We remember the last state the server accepted ("synced") and its version.
//   - A sync sends only the difference (changed items, deleted item ids, the profile row and the
//     loadout) with the version it was based on; a stale version means another device saved
//     first, so we pull and reconcile.
//   - If this device and the cloud both changed, we ask the player which one to keep.
(function () {
  var cfg = window.KZ_CLOUD || {};
  var URL_ = (cfg.url || "").replace(/\/+$/, "");
  var KEY = cfg.anonKey || "";
  var LS = { session: "kz_cloud_session", synced: "kz_cloud_synced", version: "kz_cloud_version", user: "kz_cloud_user", runs: "kz_cloud_runs" };

  // 0 off, 1 signed out, 2 connecting, 3 synced, 4 syncing, 5 offline/error, 6 conflict
  var status = URL_ && KEY ? 1 : 0;
  var session = null;
  var local = null;          // latest state from the game (object)
  var server = null;         // last fetched cloud state (for conflicts)
  var incoming = null;       // state the game should load (string), when it should adopt the cloud
  var pushTimer = null, retryTimer = null, busy = false;
  // In-game feedback for the sign-in screen: text and kind (0 info, 1 error, 2 success).
  var message = "", messageKind = 0, pending = false, recovery = false, username = "";
  function say(text, kind) { message = text || ""; messageKind = kind || 0; }
  // Turns Supabase Auth errors into player-facing sentences.
  function friendly(body, status) {
    var code = (body && (body.error_code || body.code || body.error)) || "";
    var msg = (body && (body.msg || body.message || body.error_description)) || "";
    var map = {
      invalid_credentials: "Wrong username, email or password.",
      invalid_grant: "Wrong username, email or password.",
      email_not_confirmed: "Confirm your email first: check your inbox for the link.",
      user_already_exists: "That email already has an account. Sign in instead.",
      email_exists: "That email already has an account. Sign in instead.",
      weak_password: "Choose a longer password (at least 8 characters).",
      over_email_send_rate_limit: "Too many emails sent. Try again in a few minutes.",
      over_request_rate_limit: "Too many attempts. Wait a moment and try again.",
      email_address_invalid: "That email address doesn't look right.",
      validation_failed: "Check the email address and password.",
      P0429: "Too many wrong passwords for that username. Wait 15 minutes, or sign in with your email.",
    };
    if (map[code]) return map[code];
    if (/invalid login/i.test(msg)) return map.invalid_credentials;
    if (/already registered/i.test(msg)) return map.user_already_exists;
    if (/password/i.test(msg) && /6|8|short|weak/i.test(msg)) return map.weak_password;
    return msg ? msg.charAt(0).toUpperCase() + msg.slice(1) : "Something went wrong (" + status + "). Try again.";
  }
  // Calls a database function as a signed-out visitor; errors carry PostgREST's body.
  function anonRpc(fn, args) {
    return fetch(URL_ + "/rest/v1/rpc/" + fn, { method: "POST", headers: { apikey: KEY, "Content-Type": "application/json" }, body: JSON.stringify(args) })
      .then(function (r) { return r.json().catch(function () { return null; }).then(function (j) { if (!r.ok) throw { body: j || {}, status: r.status }; return j; }); },
            function () { throw { body: { msg: "Can't reach the server. Check your connection." }, status: 0 }; });
  }
  // Whether an email has a Google account and no password (…_sign_in_methods.sql). False when
  // unknown, e.g. before that migration has been run.
  var GOOGLE_ONLY = "This email uses Google sign-in. Continue with Google below.";
  function googleOnly(email) {
    return anonRpc("sign_in_methods", { p_email: email }).then(function (m) {
      return Array.isArray(m) && m.indexOf("google") >= 0 && m.indexOf("password") < 0;
    }, function () { return false; });
  }
  function authPost(path, body) {
    pending = true;
    return fetch(URL_ + path, { method: "POST", headers: headers(true), body: JSON.stringify(body) })
      .then(function (r) { return r.text().then(function (t) { var j = t ? JSON.parse(t) : {}; pending = false; if (!r.ok) throw { body: j, status: r.status }; return j; }); })
      .catch(function (e) { pending = false; if (e && e.body !== undefined) throw e; throw { body: { msg: "Can't reach the server. Check your connection." }, status: 0 }; });
  }
  function startSession(j) {
    saveSession({ access_token: j.access_token, refresh_token: j.refresh_token, expires_at: Date.now() / 1000 + (j.expires_in || 3600), user: j.user ? { id: j.user.id, email: j.user.email || "" } : null });
    say("", 0);
    status = 2;
    if (local) reconcile();
  }
  // Google sign-in and every emailed link land on auth-callback.html, which hands the session
  // back to this tab (see the storage listener below).
  function back() { return location.origin + location.pathname.replace(/[^/]*$/, "") + "auth-callback.html"; }

  function get(k) { try { return localStorage.getItem(k); } catch (e) { return null; } }
  function set(k, v) { try { if (v === null) localStorage.removeItem(k); else localStorage.setItem(k, v); } catch (e) {} }

  // Stable JSON (sorted keys) so states compare by content.
  function canon(v) {
    if (v === null || typeof v !== "object") return JSON.stringify(v === undefined ? null : v);
    if (Array.isArray(v)) return "[" + v.map(canon).join(",") + "]";
    return "{" + Object.keys(v).sort().map(function (k) { return JSON.stringify(k) + ":" + canon(v[k]); }).join(",") + "}";
  }
  function pick(s) {
    return { profile: s.profile, items: (s.items || []).slice().sort(function (a, b) { return a.item_id - b.item_id; }), loadouts: s.loadouts || [] };
  }

  // ───────────── Auth (GoTrue REST) ─────────────
  function saveSession(s) { session = s; set(LS.session, s ? JSON.stringify(s) : null); }
  function headers(json) {
    var h = { apikey: KEY };
    if (session) h.Authorization = "Bearer " + session.access_token;
    if (json) h["Content-Type"] = "application/json";
    return h;
  }
  function fromHash() {
    var h = location.hash.replace(/^#/, "");
    if (h.indexOf("error_description=") >= 0) {
      var eq = new URLSearchParams(h);
      say(friendly({ msg: eq.get("error_description") }, 0), 1);
      history.replaceState(null, "", location.pathname + location.search);
      return false;
    }
    if (h.indexOf("access_token=") < 0) return false;
    var q = new URLSearchParams(h);
    recovery = q.get("type") === "recovery";
    saveSession({ access_token: q.get("access_token"), refresh_token: q.get("refresh_token"), expires_at: Date.now() / 1000 + Number(q.get("expires_in") || 3600) });
    history.replaceState(null, "", location.pathname + location.search);
    return true;
  }
  function refresh() {
    if (!session || !session.refresh_token) return Promise.reject(new Error("no session"));
    if (session.expires_at - 60 > Date.now() / 1000) return Promise.resolve();
    return fetch(URL_ + "/auth/v1/token?grant_type=refresh_token", { method: "POST", headers: headers(true), body: JSON.stringify({ refresh_token: session.refresh_token }) })
      .then(function (r) { if (!r.ok) throw new Error("refresh " + r.status); return r.json(); })
      .then(function (j) { saveSession({ access_token: j.access_token, refresh_token: j.refresh_token, expires_at: Date.now() / 1000 + (j.expires_in || 3600), user: j.user }); });
  }
  function loadUser() {
    return fetch(URL_ + "/auth/v1/user", { headers: headers() }).then(function (r) { if (!r.ok) throw new Error("user " + r.status); return r.json(); })
      .then(function (u) {
        session.user = { id: u.id, email: u.email || "", provider: (u.app_metadata && u.app_metadata.provider) || "email", since: u.created_at || "" };
        saveSession(session);
      });
  }
  function rpc(name, args) {
    return refresh().then(function () {
      return fetch(URL_ + "/rest/v1/rpc/" + name, { method: "POST", headers: headers(true), body: JSON.stringify(args || {}) });
    }).then(function (r) {
      return r.text().then(function (t) {
        var body = t ? JSON.parse(t) : null;
        if (!r.ok) { var e = new Error((body && body.message) || ("rpc " + r.status)); e.code = body && body.code; throw e; }
        return body;
      });
    });
  }

  // ───────────── Sync ─────────────
  function syncedState() { var s = get(LS.synced); return s ? JSON.parse(s) : null; }
  function markSynced(state, version) { set(LS.synced, canon(state)); set(LS.version, String(version)); set(LS.user, session && session.user ? session.user.id : null); }

  function fail(e) {
    console.warn("cloud:", e && e.message);
    status = 5;
    clearTimeout(retryTimer);
    retryTimer = setTimeout(reconcile, 15000);
  }

  function reconcile() {
    if (!session || !local || busy) return;
    busy = true;
    status = 2;
    var p = session.user ? Promise.resolve() : loadUser();
    p.then(function () { return rpc("get_state"); }).then(function (st) {
      busy = false;
      var mine = pick(local);
      username = st.username || username;
      if (!st.exists) return push(true, Number(st.version || 0), null);
      server = st;
      username = st.username || "";
      var theirs = pick(st);
      var sameUser = get(LS.user) === session.user.id;
      var base = syncedState();
      if (canon(mine) === canon(theirs)) { markSynced(theirs, st.version); status = 3; return flushRuns(); }
      if (sameUser && base && Number(get(LS.version)) === st.version) return push(false, st.version, base);   // only we changed
      var untouched = sameUser && base && canon(mine) === canon(base);
      var fresh = !sameUser && Number((mine.profile.doc || {}).runs || 0) === 0;
      if (untouched || fresh) return adopt(st);                                                       // only the cloud changed
      status = 6;                                                                                      // both changed: ask
    }).catch(function (e) { busy = false; fail(e); });
  }

  function adopt(st) {
    var theirs = pick(st);
    markSynced(theirs, st.version);
    incoming = JSON.stringify(theirs);
    status = 3;
    return flushRuns();
  }

  // Sends the difference between `base` (null = everything) and the current local state.
  function push(full, version, base) {
    if (!session || !local) return;
    busy = true;
    status = 4;
    var mine = pick(local);
    var before = {};
    if (base && !full) base.items.forEach(function (i) { before[i.item_id] = canon(i); });
    var upsert = mine.items.filter(function (i) { return before[i.item_id] !== canon(i); });
    var keep = {};
    mine.items.forEach(function (i) { keep[i.item_id] = true; });
    var gone = [];
    var old = full ? (server ? server.items : []) : (base ? base.items : []);
    (old || []).forEach(function (i) { if (!keep[i.item_id]) gone.push(i.item_id); });
    return rpc("sync_state", { p_base_version: version, p_profile: mine.profile, p_upsert: upsert, p_delete: gone, p_loadouts: mine.loadouts, p_reason: "sync" })
      .then(function (v) { busy = false; markSynced(mine, v); status = 3; return flushRuns(); })
      .catch(function (e) {
        busy = false;
        if (e.code === "P0409") return reconcile();   // another device saved first
        fail(e);
      });
  }

  function schedulePush() {
    clearTimeout(pushTimer);
    pushTimer = setTimeout(function () {
      if (status !== 3 && status !== 5) return;
      var base = syncedState();
      if (base && canon(pick(local)) === canon(base)) return;
      if (status === 5) return reconcile();
      push(false, Number(get(LS.version) || 0), base);
    }, 2000);
  }

  function flushRuns() {
    var q = JSON.parse(get(LS.runs) || "[]");
    if (!q.length || !session) return;
    rpc("record_run", { p_run: q[0] }).then(function () {
      q.shift();
      set(LS.runs, JSON.stringify(q));
      if (q.length) flushRuns();
    }).catch(function (e) { console.warn("cloud run:", e.message); });
  }

  // ───────────── Start-up ─────────────
  if (status) {
    var hadHash = fromHash();
    var s = get(LS.session);
    if (!session && s) { try { session = JSON.parse(s); } catch (e) {} }
    if (hadHash) session.user = null;
    if (session) status = 2;
    recovery = recovery || get("kz_cloud_recovery") === "1";
    window.addEventListener("online", function () { if (status === 5) reconcile(); });
    // A sign-in finished in another tab or window (Google popup, an emailed link).
    window.addEventListener("storage", function (e) {
      if (e.key !== LS.session || !e.newValue) return;
      try { session = JSON.parse(e.newValue); } catch (err) { return; }
      session.user = null;
      recovery = get("kz_cloud_recovery") === "1";
      say("", 0);
      status = 2;
      if (local) reconcile();
    });
  }

  // ───────────── Bridge to the game ─────────────
  var outText = new Uint8Array(0), outUser = new Uint8Array(0), outMsg = new Uint8Array(0), outAcct = new Uint8Array(0);
  function readStr(ptr, len) { return new TextDecoder().decode(new Uint8Array(wasm_memory.buffer, ptr, len)); }
  miniquad_add_plugin({
    name: "kz_cloud",
    version: 1,
    register_plugin: function (importObject) {
      var env = importObject.env;
      env.kz_cloud_status = function () { return status; };
      // Sign-in options to offer: 1 Google, 2 Discord, 4 email link.
      env.kz_cloud_providers = function () {
        var p = cfg.providers || ["google", "discord", "email"];
        return (p.indexOf("google") >= 0 ? 1 : 0) | (p.indexOf("discord") >= 0 ? 2 : 0) | (p.indexOf("email") >= 0 ? 4 : 0);
      };
      env.kz_cloud_push = function (ptr, len) {
        try { local = JSON.parse(readStr(ptr, len)); } catch (e) { return; }
        if (status === 2 && !busy) reconcile();
        else schedulePush();
      };
      env.kz_cloud_run = function (ptr, len) {
        var q = JSON.parse(get(LS.runs) || "[]");
        q.push(JSON.parse(readStr(ptr, len)));
        set(LS.runs, JSON.stringify(q.slice(-200)));
        if (status === 3) flushRuns();
      };
      env.kz_cloud_sign_in = function (ptr, len) {
        if (!status) return;
        var provider = readStr(ptr, len);
        var url = URL_ + "/auth/v1/authorize?provider=" + encodeURIComponent(provider) + "&redirect_to=" + encodeURIComponent(back());
        // A sign-in window; this tab signs in by itself when it finishes. Blocked popups fall back
        // to signing in in this tab.
        var w = 480, h = 640, x = (screen.width - w) / 2, y = (screen.height - h) / 2;
        var popup = window.open(url, "kz_sign_in", "popup,width=" + w + ",height=" + h + ",left=" + x + ",top=" + y);
        if (!popup) { location.href = url; return; }
        say("Finish signing in with Google in the window that opened.", 0);
      };
      env.kz_cloud_password_sign_in = function (ep, el, pp, pl) {
        if (!status || pending) return;
        var who = readStr(ep, el).trim(), pw = readStr(pp, pl);
        if (!who || !pw) return say("Enter your username or email and your password.", 1);
        // A username is swapped for its email by the database, which only answers when the
        // password is right (supabase/migrations/…_username_sign_in.sql).
        var email = who.indexOf("@") >= 0 ? Promise.resolve(who) : anonRpc("email_for_sign_in", { p_username: who, p_password: pw }).then(function (m) {
          if (!m) throw { body: { msg: "Wrong username or password." }, status: 400 };
          return m;
        });
        var used = "";
        pending = true;
        email.then(function (m) { used = m; return authPost("/auth/v1/token?grant_type=password", { email: m, password: pw }); })
          .then(startSession).catch(function (e) {
            pending = false;
            var code = e && e.body && (e.body.error_code || e.body.error);
            // A wrong password for an email that only has Google: point at the Google button.
            if (used && (code === "invalid_credentials" || code === "invalid_grant")) {
              return googleOnly(used).then(function (g) { say(g ? GOOGLE_ONLY : friendly(e.body, e.status), g ? 3 : 1); });
            }
            say(friendly(e && e.body, e && e.status), 1);
          });
      };
      env.kz_cloud_sign_up = function (up, ul, ep, el, pp, pl) {
        if (!status || pending) return;
        var name = readStr(up, ul).trim(), email = readStr(ep, el).trim(), pw = readStr(pp, pl);
        if (!/^[A-Za-z0-9_]{3,16}$/.test(name)) return say("Usernames are 3 to 16 letters, numbers or underscores.", 1);
        if (pw.length < 8) return say("Choose a longer password (at least 8 characters).", 1);
        pending = true;
        fetch(URL_ + "/rest/v1/rpc/username_available", { method: "POST", headers: { apikey: KEY, "Content-Type": "application/json" }, body: JSON.stringify({ p_username: name }) })
          .then(function (r) { return r.json(); })
          .then(function (free) {
            pending = false;
            if (free !== true) return say("That username is taken. Try another.", 1);
            return authPost("/auth/v1/signup?redirect_to=" + encodeURIComponent(back()), { email: email, password: pw, data: { username: name } }).then(function (j) {
              if (j.access_token) return startSession(j);
              // Supabase answers an already-registered email with a stand-in user that has no
              // identities (so emails can't be probed) and sets no password.
              var ids = j.identities || (j.user && j.user.identities);
              if (ids && ids.length === 0) {
                return googleOnly(email).then(function (g) {
                  if (g) say(GOOGLE_ONLY, 3);
                  else say("That email already has an account. Sign in instead, or use Forgot password.", 1);
                });
              }
              say("Almost there: we sent a confirmation link to " + email + ". Open it, then sign in.", 2);
            });
          })
          .catch(function (e) { pending = false; say(friendly(e && e.body, e && e.status), 1); });
      };
      env.kz_cloud_magic_link = function (ep, el) {
        if (!status || pending) return;
        var email = readStr(ep, el).trim();
        if (email.indexOf("@") < 0) return say("Type your account's email above (not your username) and try again.", 1);
        authPost("/auth/v1/otp?redirect_to=" + encodeURIComponent(back()), { email: email, create_user: true })
          .then(function () { say("Sign-in link sent to " + email + ". Open it on this device.", 2); })
          .catch(function (e) { say(friendly(e.body, e.status), 1); });
      };
      env.kz_cloud_reset = function (ep, el) {
        if (!status || pending) return;
        var email = readStr(ep, el).trim();
        if (email.indexOf("@") < 0) return say("Type your account's email above (not your username) and try again.", 1);
        authPost("/auth/v1/recover?redirect_to=" + encodeURIComponent(back()), { email: email })
          .then(function () { say("If that email has an account, a reset link is on its way.", 2); })
          .catch(function (e) { say(friendly(e.body, e.status), 1); });
      };
      env.kz_cloud_set_password = function (pp, pl) {
        if (!session || pending) return;
        var pw = readStr(pp, pl);
        if (pw.length < 8) return say("Choose a longer password (at least 8 characters).", 1);
        pending = true;
        refresh().then(function () {
          return fetch(URL_ + "/auth/v1/user", { method: "PUT", headers: headers(true), body: JSON.stringify({ password: pw }) });
        }).then(function (r) { return r.json().then(function (j) { pending = false; if (!r.ok) throw { body: j, status: r.status }; }); })
          .then(function () { recovery = false; set("kz_cloud_recovery", null); say("Password updated.", 2); })
          .catch(function (e) { pending = false; say(friendly(e && e.body, e && e.status), 1); });
      };
      env.kz_cloud_recovery = function () { return recovery ? 1 : 0; };
      // Account details for the Account panel: "username\nemail\nprovider\nmember since".
      env.kz_cloud_account_len = function () {
        var u = (session && session.user) || {};
        outAcct = new TextEncoder().encode([username, u.email || "", u.provider || "", (u.since || "").slice(0, 10)].join("\n"));
        return outAcct.length;
      };
      env.kz_cloud_account = function (ptr, len) { new Uint8Array(wasm_memory.buffer, ptr, len).set(outAcct.subarray(0, len)); };
      env.kz_cloud_pending = function () { return pending || busy ? 1 : 0; };
      env.kz_cloud_message_kind = function () { return messageKind; };
      env.kz_cloud_message_len = function () { outMsg = new TextEncoder().encode(message); return outMsg.length; };
      env.kz_cloud_message = function (ptr, len) { new Uint8Array(wasm_memory.buffer, ptr, len).set(outMsg.subarray(0, len)); };
      env.kz_cloud_clear_message = function () { say("", 0); };
      env.kz_cloud_sign_out = function () {
        if (session) fetch(URL_ + "/auth/v1/logout", { method: "POST", headers: headers() }).catch(function () {});
        saveSession(null);
        username = "";
        recovery = false;
        set("kz_cloud_recovery", null);
        [LS.synced, LS.version, LS.user].forEach(function (k) { set(k, null); });
        status = status ? 1 : 0;
      };
      // 0 = keep the cloud save, 1 = keep this device's save.
      env.kz_cloud_resolve = function (choice) {
        if (status !== 6 || !server) return;
        if (choice === 0) adopt(server);
        else push(true, server.version, null);
      };
      env.kz_cloud_incoming_len = function () {
        if (!incoming) return 0;
        outText = new TextEncoder().encode(incoming);
        incoming = null;
        return outText.length;
      };
      env.kz_cloud_incoming = function (ptr, len) { new Uint8Array(wasm_memory.buffer, ptr, len).set(outText.subarray(0, len)); };
      env.kz_cloud_user_len = function () {
        outUser = new TextEncoder().encode(username || (session && session.user ? session.user.email || "" : ""));
        return outUser.length;
      };
      env.kz_cloud_user = function (ptr, len) { new Uint8Array(wasm_memory.buffer, ptr, len).set(outUser.subarray(0, len)); };
    },
  });
})();
