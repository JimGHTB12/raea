// The DOM-like surface host elements present to the libraries React apps use (docs/ts-native.feature P18; what they
// call is docs/ts-native/reach.json). Not a browser DOM: a mirror of the host tree in JS whose nodes are EventTargets
// with attributes, queries, geometry and focus, plus window and document, the three observers, matchMedia,
// requestAnimationFrame and getComputedStyle, each as Chrome behaves for what Radix, React Aria, TanStack Virtual and
// react-hook-form (and their tests) touch.
//
// How it stands on the host:
//   - Each React host instance is an Element here (react.ts's DOM mode): its wire id is `__wid`, the tree is mirrored
//     as React commits, and its props become attributes as React DOM would set them. document.createElement makes an
//     element that goes onto the wire once it is connected (inserted under a connected, wired parent).
//   - Geometry is read synchronously from a layout snapshot the host pushes after bevy_ui's layout each frame
//     (runtime/js/src/dom.rs, `__ui_layout`). An element made this frame has no box until the next frame.
//   - Events: the host delivers an input event to the nearest element that listens for it (src/host.rs `collect`).
//     Here it becomes DOM events dispatched through real capture and bubble paths: the target is the deepest element
//     under the pointer (hit-tested against the snapshot), the path runs up to document and window, and React's own
//     handlers run as React DOM runs them, from listeners on its root container (and portal containers), along the
//     fiber tree. A press is pointerdown, mousedown, focus, pointerup, mouseup, click, as in Chrome.
//   - ResizeObserver and IntersectionObserver report from the snapshot, MutationObserver from the mirror's mutations,
//     requestAnimationFrame runs once a frame after the snapshot.
//
// Left out on purpose: innerHTML parsing, Range and Selection (a stub), shadow DOM (ShadowRoot exists for instanceof
// checks only), CSSOM stylesheets (a <style> element is kept in head, never applied), layout of elements that aren't
// connected, navigation (links and form submission do nothing past their events), iframes and windows.
export { setRunner, apiMisses, EventTarget_, Event_, CustomEvent_, UIEvent_, FocusEvent_, MouseEvent_, PointerEvent_, WheelEvent_, KeyboardEvent_, InputEvent_, SubmitEvent_, AnimationEvent_, TransitionEvent_, TouchEvent_, CompositionEvent_, ClipboardEvent_, DragEvent_, Node_, Text_, Comment_, DocumentFragment_, ShadowRoot_, Element_, HTMLElement_, focusable, HTMLInputElement_, HTMLTextAreaElement_, HTMLButtonElement_, HTMLSelectElement_, HTMLOptionElement_, HTMLFormElement_, HTMLAnchorElement_, HTMLLabelElement_, HTMLDivElement_, HTMLSpanElement_, HTMLUListElement_, HTMLLIElement_, setPageUrl, HTMLImageElement_, HTMLMediaElement_, HTMLStyleElement_, HTMLBodyElement_, HTMLHtmlElement_, HTMLHeadElement_, HTMLIFrameElement_, HTMLDialogElement_, HTMLCanvasElement_, SVGElement_, SVGSVGElement_, hostElement, hostText, hostUpdate, mirrorPlace, mirrorRemove, mirrorText, mirrorClear, listenFor, reactContainer, getComputedStyle, DOMRectReadOnly_, DOMRect_, NodeFilter_, TreeWalker_, Document_, MutationObserver_, ResizeObserver_, IntersectionObserver_, matchMedia, requestAnimationFrame, cancelAnimationFrame, effStyle, adoptDocument, setSheet, container, resetMount, layout, install } from "./dom/all";


