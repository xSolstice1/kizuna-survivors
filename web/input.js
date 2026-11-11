// In-game text fields (crates/client/src/textinput.rs). The game draws each field's box;
// a real <input> sits exactly on top of it, so phones open their keyboard, password managers
// and autofill work, and paste/IME behave. Fields the game didn't place this frame are hidden.
(function () {
  var fields = {};          // id -> { el, seen }
  var submitted = -1;       // id of a field where Enter was pressed
  var layer = null;
  var KINDS = [
    { type: "text", auto: "username" },
    { type: "email", auto: "email" },
    { type: "password", auto: "current-password" },
    { type: "password", auto: "new-password" },
  ];

  function ensureLayer() {
    if (layer) return layer;
    var style = document.createElement("style");
    style.textContent =
      "@font-face { font-family: KzNunito; src: url(Nunito-Bold.ttf); font-weight: 700; }" +
      ".kz-field { position: fixed; z-index: 5; box-sizing: border-box; margin: 0; padding: 0 0.6em;" +
      " border: 0; outline: 0; background: transparent; color: #f5f0ff; caret-color: #ffd16b;" +
      " font-family: KzNunito, system-ui, sans-serif; font-weight: 700; -webkit-user-select: text; user-select: text; }" +
      ".kz-field::placeholder { color: rgba(212, 202, 238, 0.45); }";
    document.head.appendChild(style);
    layer = document.body;
    return layer;
  }

  function make(id, kind) {
    ensureLayer();
    var el = document.createElement("input");
    var k = KINDS[kind] || KINDS[0];
    el.type = k.type;
    el.autocomplete = k.auto;
    el.className = "kz-field";
    el.spellcheck = false;
    el.setAttribute("autocapitalize", "off");
    el.setAttribute("autocorrect", "off");
    el.addEventListener("keydown", function (e) {
      if (e.key === "Enter") { submitted = id; e.preventDefault(); }
      if (e.key === "Tab") {
        // Move focus now, not on the game's next frame, so fast typing lands in the right box.
        e.preventDefault();
        var shown = Object.keys(fields).map(function (k) { return fields[k]; })
          .filter(function (f) { return f.el.style.display !== "none"; })
          .sort(function (a, b) { return (parseFloat(a.el.style.top) - parseFloat(b.el.style.top)) || (parseFloat(a.el.style.left) - parseFloat(b.el.style.left)); });
        var i = shown.indexOf(fields[id]);
        var next = shown[(i + (e.shiftKey ? shown.length - 1 : 1)) % shown.length];
        if (next) next.el.focus();
      }
      e.stopPropagation();
    });
    // miniquad swallows paste on window; keep it inside the field.
    el.addEventListener("paste", function (e) { e.stopPropagation(); });
    el.addEventListener("copy", function (e) { e.stopPropagation(); });
    el.addEventListener("cut", function (e) { e.stopPropagation(); });
    layer.appendChild(el);
    return { el: el, seen: false, kind: kind };
  }

  function readStr(ptr, len) { return new TextDecoder().decode(new Uint8Array(wasm_memory.buffer, ptr, len)); }
  var out = new Uint8Array(0);

  miniquad_add_plugin({
    name: "kz_input",
    version: 1,
    register_plugin: function (importObject) {
      var env = importObject.env;
      env.kz_input_begin = function () { for (var id in fields) fields[id].seen = false; };
      // Places field `id` (kind: 0 text, 1 email, 2 password, 3 new password) at x, y, w, h in
      // canvas CSS pixels, with a font size and placeholder.
      env.kz_input_field = function (id, kind, x, y, w, h, size, ph_ptr, ph_len) {
        var f = fields[id];
        if (!f || f.kind !== kind) {
          if (f) f.el.remove();
          f = fields[id] = make(id, kind);
        }
        f.seen = true;
        var c = document.getElementById("glcanvas").getBoundingClientRect();
        var s = f.el.style;
        s.display = "block";
        s.left = (c.left + x) + "px";
        s.top = (c.top + y) + "px";
        s.width = w + "px";
        s.height = h + "px";
        s.fontSize = size + "px";
        var ph = readStr(ph_ptr, ph_len);
        if (f.el.placeholder !== ph) f.el.placeholder = ph;
      };
      env.kz_input_end = function () {
        for (var id in fields) {
          var el = fields[id].el;
          if (fields[id].seen || el.style.display === "none") continue;
          // Hidden fields drop what was typed, so passwords don't linger in the page.
          el.style.display = "none";
          el.value = "";
          if (document.activeElement === el) el.blur();
        }
      };
      env.kz_input_focus = function (id) { if (fields[id]) fields[id].el.focus(); };
      env.kz_input_focused = function () {
        for (var id in fields) if (document.activeElement === fields[id].el) return Number(id);
        return -1;
      };
      env.kz_input_get_len = function (id) {
        out = new TextEncoder().encode(fields[id] ? fields[id].el.value : "");
        return out.length;
      };
      env.kz_input_get = function (ptr, len) { new Uint8Array(wasm_memory.buffer, ptr, len).set(out.subarray(0, len)); };
      env.kz_input_set = function (id, ptr, len) { if (fields[id]) fields[id].el.value = readStr(ptr, len); };
      env.kz_input_submitted = function () { var s = submitted; submitted = -1; return s; };
    },
  });
})();
