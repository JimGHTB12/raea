// contenteditable as Chrome edits it (docs/ts-native.feature P14a): the editing host and isContentEditable, and a
// key's or the pointer's default action in one: keypress, beforeinput (cancellable), the DOM changed in place (real
// mutation records: characterData with its old value, childList), then input. ProseMirror reads those back, so typing,
// Backspace, Delete, arrows and the pointer work as in Chrome, and what it handles itself (Enter, Mod-B) it prevents.
import * as wire from "../wire";
import { ClipboardEvent_, Element_, InputEvent_, KeyboardEvent_, Node_, S, Text_, blockAt, blockOf, charStep, clearGoal, dispatch, doc, g, lineEdge, lineStep, modify, moveTo, nodeLength, selection, syncListens, textsIn, wordAt, wordStep } from "./all";

type Pos = [Node_, number];

// ---------------------------------------------------------------- the editing host

const EDITABLE = new Set(["", "true", "plaintext-only"]);

/** an element's own contenteditable: true, false, or inherited (null) */
function own(el: Element_): boolean | null {
  const a = el.getAttribute("contenteditable");
  if (a === null) {
    const p = el._props.contentEditable;
    if (p === true || p === "true" || p === "") return true;
    if (p === false || p === "false") return false;
    return null;
  }
  return EDITABLE.has(a.toLowerCase()) ? true : a.toLowerCase() === "false" ? false : null;
}

export function isEditable(el: Element_): boolean {
  for (let n: Node_ | null = el; n instanceof Element_; n = n.parentNode) {
    const o = own(n);
    if (o !== null) return o;
  }
  return false;
}

/** the editing host `n` is in: the outermost editable element up from it */
export function editingHost(n: Node_ | null): Element_ | null {
  let host: Element_ | null = null;
  for (let x: Node_ | null = n; x instanceof Element_ || x instanceof Text_; x = x.parentNode) {
    if (!(x instanceof Element_)) continue;
    const o = own(x);
    if (o === true) host = x;
    else if (o === false) break;
  }
  return host;
}

/** an editing host itself (focusable, as Chrome makes one) */
export const isEditingHost = (el: Element_) => own(el) === true && !(el.parentNode instanceof Element_ && isEditable(el.parentNode));

/** contenteditable set or removed: the host's tabIndex and its keys follow */
export function editableChanged(el: Element_) {
  if (!el._wired || el.__wid === undefined) return;
  if (isEditingHost(el) && !el.hasAttribute("tabindex")) wire.set(el.__wid, "tabIndex", 0);
  syncListens(el);
}

/** the editing host the selection's focus is in, when it has the focus */
function activeHost(): Element_ | null {
  const f = selection.focusNode;
  const h = editingHost(f);
  return h && S.active && (S.active === h || S.active.contains(h)) ? h : null;
}

(globalThis as { __raea_modify?: unknown }).__raea_modify = (s: typeof selection, a: string, d: string, gr: string) => modify(s, a, d, gr, editingHost(s.focusNode));

/** A focused editing host with no selection in it: the caret at its start (Chrome's focus()) */
export function focusedHost(el: Element_) {
  if (!isEditingHost(el)) return;
  if (selection.focusNode && el.contains(selection.focusNode)) return selection._changed(true);
  const t = textsIn(el)[0];
  selection.collapse(t ?? el, 0);
}

// ---------------------------------------------------------------- the edits

/** beforeinput (cancellable), the edit, then input: Chrome's order. Returns whether the edit ran. */
function edit(host: Element_, inputType: string, data: string | null, run: () => void): boolean {
  const init = { bubbles: true, cancelable: true, composed: true, inputType, data, isComposing: false, view: g };
  if (!dispatch(host, new InputEvent_("beforeinput", init), true)) return false;
  run();
  dispatch(host, new InputEvent_("input", { ...init, cancelable: false }), true);
  return true;
}

/** the selection's contents gone, the caret where they were */
function deleteSelection() {
  const r = selection.rangeCount ? selection.getRangeAt(0) : null;
  if (!r || r.collapsed) return;
  r.deleteContents();
  selection.collapse(r.startContainer, r.startOffset);
}

/** an element's own string (React's text content) as a real text child, so an edit has a node to change */
function ownText(el: Element_) {
  if (el._inline === undefined) return;
  const s = el._inline;
  el.textContent = "";
  el._inline = undefined;
  if (el._wired && el.__wid !== undefined) wire.text(el.__wid, "");
  el.appendChild(doc.createTextNode(s));
}

/** text put in at the caret: into the text it's in or beside, else a new text (an empty block's `<br>` replaced) */
export function insertText(s: string) {
  deleteSelection();
  let n = selection.focusNode;
  let o = selection.focusOffset;
  if (!n) return;
  if (n instanceof Element_ && n._inline !== undefined) {
    const at = o ? n._inline.length : 0;
    ownText(n);
    [n, o] = [n.firstChild!, at];
  }
  if (n instanceof Text_) {
    n.insertData(o, s);
    selection.collapse(n, o + s.length);
    return;
  }
  const before = n._kids[o - 1];
  const after = n._kids[o];
  if (before instanceof Text_) {
    before.appendData(s);
    selection.collapse(before, before.data.length);
  } else if (after instanceof Text_) {
    after.insertData(0, s);
    selection.collapse(after, s.length);
  } else {
    const t = doc.createTextNode(s);
    // an empty block's placeholder break goes as text comes in
    if (after instanceof Element_ && after.localName === "br" && !after.nextSibling) n.replaceChild(t, after);
    else n.insertBefore(t, after ?? null);
    selection.collapse(t, s.length);
  }
}

/** a character back or on deleted (or the selection): within a text, or two blocks joined at their edge */
function deleteChar(dir: -1 | 1, host: Element_) {
  if (!selection.isCollapsed) return deleteSelection();
  const f: Pos = [selection.focusNode!, selection.focusOffset];
  const to = charStep(f, dir, host);
  if (to[0] === f[0] && to[1] === f[1]) return;
  const [a, b] = dir < 0 ? [to, f] : [f, to];
  const ba = blockOf(a[0], host);
  const bb = blockOf(b[0], host);
  if (ba !== bb && bb !== host && ba.contains(bb) === false && bb.contains(ba) === false) {
    // the later block's contents into the earlier one, and it gone
    const keep = a;
    for (const k of bb._kids.slice()) ba.appendChild(k);
    bb.remove();
    for (const br of ba._kids.filter((k) => k instanceof Element_ && k.localName === "br" && k !== ba.lastChild)) br.remove();
    selection.collapse(keep[0], Math.min(keep[1], nodeLength(keep[0])));
    return;
  }
  selection.setBaseAndExtent(a[0], a[1], b[0], b[1]);
  deleteSelection();
}

/** Enter: the block split at the caret (a new one of the same kind after it), as Chrome's insertParagraph */
function splitBlock(host: Element_) {
  deleteSelection();
  const n = selection.focusNode!;
  const o = selection.focusOffset;
  const b = blockOf(n, host);
  if (b === host) {
    // a host with no blocks: what's after the caret into a div
    const div = doc.createElement("div");
    moveAfter(host, n, o, div);
    host.appendChild(div);
    if (!div.firstChild) div.appendChild(doc.createElement("br"));
    selection.collapse(textsIn(div)[0] ?? div, 0);
    return;
  }
  const nb = b.cloneNode(false) as Element_;
  moveAfter(b, n, o, nb);
  b.parentNode!.insertBefore(nb, b.nextSibling);
  if (!textsIn(b).some((t) => t.data)) b.appendChild(doc.createElement("br"));
  if (!nb.firstChild) nb.appendChild(doc.createElement("br"));
  selection.collapse(textsIn(nb)[0] ?? nb, 0);
}

/** what's after (n, o) in `from` moved into `to` (a text split at the point) */
function moveAfter(from: Element_, n: Node_, o: number, to: Element_) {
  let start: Node_;
  if (n instanceof Text_) start = o < n.data.length ? (o ? n.splitText(o) : n) : n.nextSibling!;
  else start = n._kids[o];
  if (!start) return;
  // up to the child of `from` the point is in
  while (start && start.parentNode !== from) start = start.parentNode!.nextSibling ?? start.parentNode!;
  for (let k: Node_ | null = start; k; ) {
    const next: Node_ | null = k.nextSibling;
    to.appendChild(k);
    k = next;
  }
}

// ---------------------------------------------------------------- clipboard (the page's own, as the window has none)

let clipboard = "";

function clipboardEvent(type: string, data: string) {
  const store = new Map<string, string>([["text/plain", data]]);
  const ev = new ClipboardEvent_(type, { bubbles: true, cancelable: true, composed: true }) as ClipboardEvent_ & { clipboardData: unknown };
  ev.clipboardData = {
    types: ["text/plain"],
    getData: (t: string) => store.get(t === "text" ? "text/plain" : t) ?? "",
    setData: (t: string, v: string) => void store.set(t === "text" ? "text/plain" : t, String(v)),
    clearData: () => store.clear(),
    items: [],
    files: [],
  };
  return { ev, store };
}

// ---------------------------------------------------------------- keys and the pointer

const mac = () => /Mac/.test(String(g.navigator?.platform ?? ""));

/** A key's default action in a focused editing host, after its keydown went unprevented. Returns whether it acted
 *  (the host's own default, a scroll, is then prevented). */
export function editKey(t: Element_, init: { key: string; shiftKey: boolean; ctrlKey: boolean; altKey: boolean; metaKey: boolean; keyCode: number }): boolean {
  const host = activeHost() ?? (isEditable(t) ? editingHost(t) : null);
  if (!host || !selection.focusNode) return false;
  const k = init.key;
  const mod = mac() ? init.metaKey : init.ctrlKey;
  const ext = init.shiftKey;
  const f: Pos = [selection.focusNode, selection.focusOffset];
  const collapseTo = (dir: -1 | 1): Pos => {
    const r = selection.getRangeAt(0);
    return dir < 0 ? [r.startContainer, r.startOffset] : [r.endContainer, r.endOffset];
  };
  const word = mac() ? init.altKey : init.ctrlKey;
  if (k !== "ArrowUp" && k !== "ArrowDown") clearGoal();
  switch (k) {
    case "ArrowLeft":
    case "ArrowRight": {
      const dir = k === "ArrowLeft" ? -1 : 1;
      if (!ext && !selection.isCollapsed && !word && !(mac() && init.metaKey)) moveTo(selection, collapseTo(dir), false);
      else moveTo(selection, mac() && init.metaKey ? lineEdge(f, dir, host) : word ? wordStep(f, dir, host) : charStep(f, dir, host), ext);
      return true;
    }
    case "ArrowUp":
    case "ArrowDown":
      moveTo(selection, lineStep(f, k === "ArrowUp" ? -1 : 1, host), ext);
      return true;
    case "Home":
    case "End":
      moveTo(selection, lineEdge(f, k === "Home" ? -1 : 1, host), ext);
      return true;
    case "Backspace":
    case "Delete": {
      const dir = k === "Backspace" ? -1 : 1;
      const type = word ? (dir < 0 ? "deleteWordBackward" : "deleteWordForward") : dir < 0 ? "deleteContentBackward" : "deleteContentForward";
      edit(host, type, null, () => {
        if (word && selection.isCollapsed) moveTo(selection, wordStep(f, dir, host), true);
        deleteChar(dir, host);
      });
      return true;
    }
    case "Enter":
      if (ext) edit(host, "insertLineBreak", null, () => insertBreak());
      else edit(host, "insertParagraph", null, () => splitBlock(host));
      return true;
    case "Tab":
      return false;
    default:
      break;
  }
  if (mod && !init.altKey) {
    const c = k.toLowerCase();
    if (c === "a") {
      const texts = textsIn(host);
      const last = texts[texts.length - 1];
      selection.setBaseAndExtent(texts[0] ?? host, 0, last ?? host, last ? last.data.length : host._kids.length);
      return true;
    }
    if (c === "c" || c === "x") {
      const { ev, store } = clipboardEvent(c === "c" ? "copy" : "cut", selection.toString());
      const ok = dispatch(host, ev, true);
      if (!ok || selection.toString()) clipboard = store.get("text/plain") ?? "";
      if (ok && c === "x" && !selection.isCollapsed) edit(host, "deleteByCut", null, () => deleteSelection());
      return true;
    }
    if (c === "v") {
      const { ev } = clipboardEvent("paste", clipboard);
      if (dispatch(host, ev, true) && clipboard) edit(host, "insertFromPaste", clipboard, () => insertText(clipboard));
      return true;
    }
    return false;
  }
  // a character typed: keypress, then the insert
  if (k.length && [...k].length === 1 && !init.ctrlKey && !init.metaKey) {
    const kp = new KeyboardEvent_("keypress", { ...init, bubbles: true, cancelable: true, composed: true, view: g, charCode: k.codePointAt(0), keyCode: k.codePointAt(0), which: k.codePointAt(0) });
    if (!dispatch(selection.focusNode instanceof Element_ ? selection.focusNode : (selection.focusNode.parentNode ?? host), kp, true)) return true;
    edit(host, "insertText", k, () => insertText(k));
    return true;
  }
  return false;
}

/** Shift+Enter: a `<br>` at the caret (and one more at a block's end, so the new line shows) */
function insertBreak() {
  deleteSelection();
  let n = selection.focusNode!;
  let o = selection.focusOffset;
  const br = doc.createElement("br");
  if (n instanceof Text_) {
    const rest = o < n.data.length ? n.splitText(o) : null;
    n.parentNode!.insertBefore(br, rest ?? n.nextSibling);
    [n, o] = [br.parentNode!, br.parentNode!._kids.indexOf(br) + 1];
  } else {
    n.insertBefore(br, n._kids[o] ?? null);
    o++;
  }
  if (!br.nextSibling) br.parentNode!.appendChild(doc.createElement("br"));
  selection.collapse(n, o);
}

let dragging = false;

/** The pointer in an editable (its mousedown unprevented): a press places the caret (Shift extends; two clicks pick
 *  a word, three its paragraph), a drag with the button down extends, the release ends it. */
export function editPointer(kind: "down" | "move" | "up", t: Element_, x: number, y: number, p: { shiftKey?: boolean; detail?: number }) {
  if (kind === "up") {
    dragging = false;
    return;
  }
  if (kind === "move") {
    if (!dragging) return;
    const at = doc.caretPositionFromPoint(x, y);
    const host = editingHost(selection.anchorNode);
    if (at && host && host.contains(at.offsetNode)) moveTo(selection, [at.offsetNode, at.offset], true);
    return;
  }
  const host = editingHost(t);
  if (!host) return;
  clearGoal();
  const at = doc.caretPositionFromPoint(x, y);
  const pos: Pos = at && host.contains(at.offsetNode) ? [at.offsetNode, at.offset] : [textsIn(host)[0] ?? host, 0];
  const clicks = p.detail ?? 1;
  if (clicks === 2 || clicks >= 3) {
    const [a, b] = clicks === 2 ? wordAt(pos, host) : blockAt(pos, host);
    selection.setBaseAndExtent(a[0], a[1], b[0], b[1]);
    return;
  }
  moveTo(selection, pos, !!p.shiftKey && !!selection.anchorNode && host.contains(selection.anchorNode));
  dragging = true;
}

/** The page can edit rich text: a rich-text editor (Tiptap) checks `__raea_editing` before it starts
 *  (set as the DOM installs) */
export function markEditing() {
  g.__raea_editing = true;
}

