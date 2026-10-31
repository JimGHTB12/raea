// raea-page: a frame's own window, while iframes are on (docs/blitz-bevy.feature B5c, host.rs's mount_frames). The
// host sets __raea_frame = { url, opaque } before this; @raea/ui's DOM calls installed() once its globals are in, before
// the frame's modules run: its URL as location, parent and top as its parent's window (postMessage, which the host
// takes to the parent's root), and an opaque origin ("null") when its sandbox has no allow-same-origin.
Object.assign(globalThis.__raea_frame, {
  installed() {
    const g = globalThis;
    location.href = this.url;
    const parent = {
      postMessage(data, origin = "*") {
        g.__ui_emit("frame.post", JSON.stringify({ data, origin: String(origin) }));
      },
      closed: false,
    };
    parent.window = parent.self = parent.frames = parent;
    for (const k of ["parent", "top"]) Object.defineProperty(g, k, { value: parent, configurable: true, writable: true });
    Object.defineProperty(g, "frameElement", { value: null, configurable: true, writable: true });
    if (this.opaque) Object.defineProperty(g, "origin", { value: "null", configurable: true, writable: true });
    this.parent = parent;
  },
  deliver(data, origin) {
    const e = new Event("message");
    Object.defineProperties(e, { data: { value: data }, origin: { value: origin }, source: { value: this.parent ?? null }, lastEventId: { value: "" }, ports: { value: [] } });
    dispatchEvent(e);
  },
});
