// contenteditable and Selection (docs/ts-native.feature P14a) on the stand-in host: typing, Backspace, Delete and
// arrows edit the DOM in place with Chrome's events and mutation records, and the selection's range is live. The
// same is proved against Chrome by sdk/ui/parity/cases/p14-forms/contenteditable.tsx; this is the fast check.
import { beforeAll, describe, expect, test } from "bun:test";
import { install } from "../src/dom";
import { restart } from "../src/wire";
import { Host } from "./host";

beforeAll(() => install());
const g = globalThis as any;
const d = () => g.document;

function editor(html: (el: any) => void) {
  restart();
  new Host();
  const el = d().createElement("div");
  el.setAttribute("contenteditable", "true");
  html(el);
  d().body.appendChild(el);
  el.focus();
  return el;
}
const key = (k: string, mods: Record<string, boolean> = {}) => g.__ui_deliver(JSON.stringify({ t: "named", name: "keydown", value: { key: k, ...mods } }));
const type = (s: string) => [...s].forEach((c) => key(c));

describe("P14a contenteditable", () => {
  test("isContentEditable, contentEditable and focus", () => {
    const el = editor((e) => e.appendChild(d().createElement("p")));
    expect(el.isContentEditable).toBe(true);
    expect(el.contentEditable).toBe("true");
    expect(el.firstChild.isContentEditable).toBe(true);
    expect(d().activeElement).toBe(el);
    const off = d().createElement("span");
    off.contentEditable = "false";
    el.appendChild(off);
    expect(off.isContentEditable).toBe(false);
  });

  test("typing inserts at the caret, with beforeinput, input and characterData records", async () => {
    const el = editor((e) => e.appendChild(d().createTextNode("ac")));
    const t = el.firstChild;
    const events: string[] = [];
    for (const ty of ["keydown", "keypress", "beforeinput", "input"]) el.addEventListener(ty, (e: any) => events.push(`${ty}:${e.inputType ?? e.key}`));
    const records: any[] = [];
    const mo = new g.MutationObserver((rs: any[]) => records.push(...rs));
    mo.observe(el, { subtree: true, characterData: true, characterDataOldValue: true, childList: true });
    g.getSelection().collapse(t, 1);
    type("b");
    expect(el.textContent).toBe("abc");
    expect(events).toEqual(["keydown:b", "keypress:b", "beforeinput:insertText", "input:insertText"]);
    const s = g.getSelection();
    expect([s.focusNode, s.focusOffset, s.isCollapsed]).toEqual([t, 2, true]);
    await Promise.resolve();
    expect(records.map((r) => [r.type, r.oldValue])).toEqual([["characterData", "ac"]]);
  });

  test("a prevented beforeinput edits nothing", () => {
    const el = editor((e) => e.appendChild(d().createTextNode("x")));
    el.addEventListener("beforeinput", (e: any) => e.preventDefault());
    g.getSelection().collapse(el.firstChild, 1);
    type("y");
    expect(el.textContent).toBe("x");
  });

  test("Backspace, Delete and arrows across text nodes and a mark", () => {
    const el = editor((e) => {
      e.appendChild(d().createTextNode("ab"));
      const b = d().createElement("strong");
      b.appendChild(d().createTextNode("cd"));
      e.appendChild(b);
      e.appendChild(d().createTextNode("ef"));
    });
    const s = g.getSelection();
    s.collapse(el.lastChild, 2);
    key("Backspace");
    expect(el.textContent).toBe("abcde");
    key("ArrowLeft");
    // from the text after the mark, a step back is into the mark (between its c and d)
    key("ArrowLeft");
    expect([s.focusNode.data, s.focusOffset]).toEqual(["cd", 1]);
    key("Delete");
    expect(el.textContent).toBe("abce");
    key("ArrowRight", { shiftKey: true });
    key("ArrowRight", { shiftKey: true });
    expect(s.toString()).toBe("e");
    type("Z");
    expect(el.textContent).toBe("abcZ");
  });

  test("Enter splits the block; Backspace at its start joins it back", () => {
    const el = editor((e) => {
      const p = d().createElement("p");
      p.appendChild(d().createTextNode("hello"));
      e.appendChild(p);
    });
    const s = g.getSelection();
    s.collapse(el.firstChild.firstChild, 2);
    key("Enter");
    expect(el.children.length).toBe(2);
    expect([el.children[0].textContent, el.children[1].textContent]).toEqual(["he", "llo"]);
    key("Backspace");
    expect(el.children.length).toBe(1);
    expect(el.textContent).toBe("hello");
  });

  test("the selection's range is live: a removed node moves it to the parent", () => {
    const el = editor((e) => {
      e.appendChild(d().createTextNode("a"));
      e.appendChild(d().createTextNode("b"));
    });
    const s = g.getSelection();
    s.collapse(el.lastChild, 1);
    el.removeChild(el.lastChild);
    expect([s.focusNode, s.focusOffset]).toEqual([el, 1]);
    const r = d().createRange();
    r.setStart(el.firstChild, 0);
    r.setEnd(el, 1);
    expect(r.toString()).toBe("a");
    expect(r.collapsed).toBe(false);
  });

  test("selectionchange comes as a task, once for several changes", async () => {
    const el = editor((e) => e.appendChild(d().createTextNode("abc")));
    let n = 0;
    d().addEventListener("selectionchange", () => n++);
    await new Promise((r) => setTimeout(r, 1));
    n = 0;
    const s = g.getSelection();
    s.collapse(el.firstChild, 0);
    s.extend(el.firstChild, 2);
    expect(n).toBe(0);
    await new Promise((r) => setTimeout(r, 1));
    expect(n).toBe(1);
  });
});
