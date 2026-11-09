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

self.addEventListener("fetch", function (e) {
  if (e.request.method !== "GET") return;
  var key = e.request.mode === "navigate" ? "./" : e.request;
  e.respondWith(caches.open(CACHE).then(function (c) {
    return c.match(key, { ignoreSearch: true }).then(function (hit) { return hit || fetch(e.request); });
  }));
});
