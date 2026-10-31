// Host nodes made, patched and dropped.
import { allocId, buildUpdateOp, dropHandlers, push, serializeProps } from "../../node_modules/bevy-react/src/bridge";
import { Host, Instance, Out, Slot, same } from "./all";

function kindOf(type: string, inText: boolean) {
  return !inText ? type : type === "text" ? "textSpan" : type === "image" ? "inlineImage" : type;
}

/** The named components an element was rendered in, nearest first, up to three ("Paddle Court Pong"). */
function ownerOf(inst: Instance | null): string {
  let names = "";
  for (let n = 0; inst && n < 3; inst = inst.parent) {
    const name = inst.fn?.displayName ?? inst.fn?.name;
    if (typeof name !== "string" || !name) continue;
    names = n++ ? `${names} ${name}` : name;
  }
  return names;
}

function create(o: Out): Host {
  const id = allocId();
  if (o.type === "#text") {
    push(o.inText ? { op: "createTextSpan", id, text: o.text } : { op: "createText", id, text: o.text });
    return { id, type: "#text", kind: "#text", text: true, props: null, value: o.text, kids: [], slot: o.slot, out: o };
  }
  const kind = kindOf(o.type, o.inText);
  const props = serializeProps(id, o.props, o.type);
  // the components it was rendered in, nearest first: what the feed's play stage names a picked element by
  if (o.owner && props) props["data-raea-owner"] = ownerOf(o.owner);
  push({ op: "create", id, kind, props, text: o.text });
  const h: Host = { id, type: o.type, kind, text: false, props: o.props, value: o.text, kids: [], slot: o.slot, out: o };
  for (const k of o.kids) {
    const c = create(k);
    push({ op: "append", parent: id, child: c.id });
    h.kids.push(c);
  }
  return h;
}

function drop(h: Host) {
  dropHandlers(h.id);
  for (const k of h.kids) drop(k);
}

function patch(h: Host, o: Out) {
  if (h.out === o) return;
  h.out = o;
  if (h.text) {
    if (h.value !== o.text) push({ op: "updateText", id: h.id, text: o.text });
    h.value = o.text;
    return;
  }
  const op = buildUpdateOp(h.id, h.props, o.props, h.type, h.kind);
  if (op) push(op);
  if (o.text !== undefined && o.text !== h.value) push({ op: "updateText", id: h.id, text: o.text });
  else if (o.text === undefined && h.value !== undefined && h.type === "text") push({ op: "updateText", id: h.id, text: "" });
  h.value = o.text;
  h.props = o.props;
  h.kids = children(h.id, h.kids, o.kids);
}

/** A host parent's children matched to what the render yields: same place and type kept (patched), the rest made or
 * removed, and moved where their order changed. */
export function children(parent: number, old: Host[], next: Out[]): Host[] {
  // the usual frame: the same places in the same order, each patched where it is (no maps, nothing moves)
  if (old.length === next.length) {
    let same = true;
    for (let i = 0; i < old.length; i++) {
      const h = old[i];
      const o = next[i];
      if (h.slot !== o.slot || h.type !== o.type || !(h.text || h.kind === kindOf(o.type, o.inText))) {
        same = false;
        break;
      }
    }
    if (same) {
      for (let i = 0; i < old.length; i++) patch(old[i], next[i]);
      return old;
    }
  }
  const bySlot = new Map<Slot, Host>();
  for (const h of old) bySlot.set(h.slot, h);
  const kept = new Set<Host>();
  const out: Host[] = next.map((o) => {
    const h = bySlot.get(o.slot);
    if (h && h.type === o.type && !kept.has(h) && (h.text || h.kind === kindOf(o.type, o.inText))) {
      kept.add(h);
      patch(h, o);
      return h;
    }
    return create(o);
  });
  for (const h of old) {
    if (!kept.has(h)) {
      push({ op: "remove", parent, child: h.id });
      drop(h);
    }
  }
  // order: walk from the end, each one in place before the next (only those out of place move)
  const pos = new Map<Host, number>();
  old.forEach((h, i) => kept.has(h) && pos.set(h, i));
  let lastOld = Infinity;
  for (let i = out.length - 1; i >= 0; i--) {
    const h = out[i];
    const was = pos.get(h);
    if (was !== undefined && was < lastOld) {
      lastOld = was;
      continue;
    }
    const before = out[i + 1];
    push(before ? { op: "insert", parent, child: h.id, before: before.id } : { op: "append", parent, child: h.id });
  }
  return out;
}
