// Portions ported from HyperFrames (Apache-2.0, https://github.com/heygen-com/hyperframes); see NOTICE.
// An HTML composition on bevy_ui (docs/ts-native.feature P27): an HTML composition page (HTML, CSS, GSAP and the
// page's own scripts) run in a raea-runtime-js root, its elements dom.ts's, so its scripts work on a DOM and every element is
// a bevy_ui entity. The host (runtime/js src/html_page.rs) mounts this bundle with `__html` set (the page's
// source, its file and folder, the folder's files) and drives it a frame at a time:
//
//   load:   the page parsed, compiled (sub-compositions inlined: html/compile.ts), built into dom.ts elements under
//           html and body on the host, its stylesheets fetched and cascaded (html/cascade.ts), its scripts run in
//           order; then `html.ready` {w, h, duration, videos, audio, warnings} is emitted.
//   seek:   the named event `html.seek` {t}: the page clock (Date.now and performance.now, which GSAP's ticker reads)
//           set to t, GSAP's global timeline ticked, the registered timelines (`__timelines`, sub-compositions'
//           nested at their host's start) moved to t, clips shown and hidden by their windows (html/runtime.ts),
//           videos seeked, the cascade redone if the page changed; then `html.seeked` {t}.
//
// Files and the network go through the host call `html.read` {url, binary?} -> {path, text?}: a project-relative
// path or an http(s) URL, fetched once into raea's cache (the same one engine/html's SyncNet kept).
import "./dom-globals";
import "./html/page/all";


