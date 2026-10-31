// raea-page: a page's <iframe>s' windows, while iframes are on (docs/blitz-bevy.feature B5c, host.rs's frames). An
// iframe's contentWindow (@raea/ui asks __raea_frames.window) is a cross-origin window: postMessage, which the host
// takes to the frame's root, and itself as window, self and frames. A frame's message comes back as a message event on
// this page's window, its source that same object, so `e.source === iframe.contentWindow` holds.
globalThis.__raea_frames = (() => {
  const wins = new Map();
  const win = (wid) => {
    let w = wins.get(wid);
    if (!w) {
      w = {
        postMessage(data, origin = "*") {
          globalThis.__ui_emit("frame.postTo", JSON.stringify({ frame: wid, data, origin: String(origin) }));
        },
        closed: false,
      };
      w.window = w.self = w.frames = w;
      wins.set(wid, w);
    }
    return w;
  };
  return {
    window: (el) => (el.__wid === undefined ? null : win(el.__wid)),
    deliver(wid, data, origin) {
      const e = new Event("message");
      Object.defineProperties(e, { data: { value: data }, origin: { value: origin }, source: { value: win(wid) }, lastEventId: { value: "" }, ports: { value: [] } });
      dispatchEvent(e);
    },
  };
})();
