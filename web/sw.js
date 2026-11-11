// Offline play: cache the whole game on install, then serve it from the cache.
// scripts/build-web.sh fills in __BUILD__ (a hash of dist/) and __FILES__ (every file in dist/),
// so each deploy gets a fresh cache and the old one is dropped.
var CACHE = "kizuna-__BUILD__";
var FILES = ["./", __FILES__];

self.addEventListener("install", function (e) {
  e.waitUntil(caches.open(CACHE).then(function (c) { return c.addAll(FILES); }).then(function () { return self.skipWaiting(); }));
});

self.addEventListener("activate", function (e) {
  e.waitUntil(caches.keys().then(function (keys) {
    return Promise.all(keys.filter(function (k) { return k !== CACHE; }).map(function (k) { return caches.delete(k); }));
  }).then(function () { return self.clients.claim(); }));
});

// A stale cached build must not run once and resave the profile, so the page and the game's code
// try the network first and only fall back to the cache offline.
function networkFirst(path, mode) {
  return mode === "navigate" || /\.(wasm|js)$/.test(path);
}

// A stalled connection never rejects, so give up on it after a few seconds and try the cache.
var NETWORK_TIMEOUT_MS = 4000;

function timeout(p, ms) {
  return new Promise(function (resolve, reject) {
    var t = setTimeout(reject, ms);
    p.then(function (v) { clearTimeout(t); resolve(v); }, function (err) { clearTimeout(t); reject(err); });
  });
}

self.addEventListener("fetch", function (e) {
  if (e.request.method !== "GET") return;
  var key = e.request.mode === "navigate" ? "./" : e.request;
  var fromCache = function (c) { return c.match(key, { ignoreSearch: true }); };
  if (networkFirst(new URL(e.request.url).pathname, e.request.mode)) {
    var net = fetch(e.request).then(function (res) {
      if (!res.ok) return res;
      var copy = res.clone();
      caches.open(CACHE).then(function (c) { return c.put(key, copy); });
      return res;
    });
    e.respondWith(timeout(net, NETWORK_TIMEOUT_MS).catch(function () {
      return caches.open(CACHE).then(fromCache).then(function (hit) { return hit || net; });
    }));
    return;
  }
  e.respondWith(caches.open(CACHE).then(function (c) {
    return fromCache(c).then(function (hit) { return hit || fetch(e.request); });
  }));
});
