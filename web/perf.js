// Mobile power savers, loaded before miniquad (mq_js_bundle.js) reads the browser.
(function (w) {
  // Render at most 2x pixel density: 3x phones would otherwise draw ~2.25x the pixels for
  // text that already looks sharp at 2x.
  var dpr = w.devicePixelRatio || 1;
  try {
    Object.defineProperty(w, "devicePixelRatio", { configurable: true, get: function () { return Math.min(dpr, 2); } });
  } catch (e) {}

  // Phones and tablets: cap frames at 60 fps to save battery. Desktops run at the display's
  // own rate; the client draws between sim steps, so 120/144 Hz screens get smooth motion.
  // Frames are skipped, not delayed, so a slow device never catches up in a burst.
  if (!(w.matchMedia && w.matchMedia("(pointer: coarse)").matches)) return;
  var raf = w.requestAnimationFrame.bind(w);
  var STEP = 1000 / 60, next = 0;
  w.requestAnimationFrame = function (cb) {
    return raf(function tick(t) {
      if (t < next - 1) return raf(tick);
      next = Math.max(next + STEP, t);
      cb(t);
    });
  };
})(window);
