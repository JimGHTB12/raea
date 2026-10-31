// P18: the tree as the libraries query it: querySelector(All), closest, contains, matches, getElementById, the
// document's body and documentElement, ownerDocument and defaultView, isConnected, document.createElement and
// appendChild/removeChild, instanceof the element classes, compareDocumentPosition and a TreeWalker over focusables.
// proves: api.Element.querySelectorAll, api.Element.querySelector, api.Document.querySelectorAll, api.Document.querySelector, api.Element.closest, api.Node.contains, api.Element.matches, api.Document.getElementById, api.Document.body, api.Document.documentElement, api.Node.ownerDocument, api.Document.defaultView, api.Node.isConnected, api.Document.createElement, api.Node.appendChild, api.Node.removeChild, api.HTMLElement, api.Element, api.Node, api.Node.compareDocumentPosition, api.Document.createTreeWalker, api.TreeWalker.nextNode, api.Node.parentNode, api.Node.parentElement, api.Element.children, api.Node.childNodes, api.Node.firstChild, api.Node.nextSibling
// tier: quick
import { useEffect, useRef } from "react";
import { trace } from "parity";

export const steps = [];

const tid = (e: Element | null) => e?.getAttribute("data-testid") ?? null;

export default function Case() {
  const list = useRef<HTMLUListElement>(null);
  const slot = useRef<HTMLDivElement>(null);
  useEffect(() => {
    const l = list.current!;
    trace("all", "list", [...l.querySelectorAll("li")].map(tid));
    trace("sel", "list", [tid(l.querySelector("li.on")), tid(l.querySelector("[data-n='2']")), tid(l.querySelector("li:not(.on) + li")), tid(l.querySelector(":scope > li:last-child")), l.querySelectorAll("button, [tabindex]").length]);
    trace("doc", "", [document.querySelectorAll("[data-testid]").length, tid(document.querySelector("#the-list li")), tid(document.getElementById("the-list")), document.getElementById("nope")]);
    const b = l.querySelector("button")!;
    trace("closest", "b", [tid(b.closest("li")), tid(b.closest("ul")), b.closest("table"), b.matches("li > button"), b.matches(".x")]);
    trace("contains", "", [l.contains(b), b.contains(l), l.contains(l), document.body.contains(l), document.documentElement.contains(document.body)]);
    trace("doc-props", "", [document.body.tagName, document.documentElement.tagName, l.ownerDocument === document, l.ownerDocument.defaultView === window, l.isConnected]);
    trace("instanceof", "", [l instanceof HTMLElement, l instanceof Element, l instanceof Node, b instanceof HTMLButtonElement, b instanceof HTMLInputElement, document.createTextNode("x") instanceof Element]);
    const first = l.firstElementChild!;
    trace("order", "", [first.compareDocumentPosition(b) & Node.DOCUMENT_POSITION_FOLLOWING, b.compareDocumentPosition(first) & Node.DOCUMENT_POSITION_PRECEDING, l.compareDocumentPosition(b) & Node.DOCUMENT_POSITION_CONTAINED_BY]);
    trace("tree", "", [tid(b.parentNode as Element), tid(b.parentElement), l.children.length, first.childNodes.length, first.firstChild?.nodeType, tid(first.nextElementSibling), first.nextSibling === first.nextElementSibling]);
    const w = document.createTreeWalker(l, NodeFilter.SHOW_ELEMENT, { acceptNode: (n) => ((n as Element).matches("button, [tabindex]") ? NodeFilter.FILTER_ACCEPT : NodeFilter.FILTER_SKIP) });
    const seen: (string | null)[] = [];
    for (let n = w.nextNode(); n; n = w.nextNode()) seen.push(tid(n as Element));
    trace("walker", "", seen);
    const made = document.createElement("div");
    made.setAttribute("data-testid", "made");
    trace("made", "", [made.isConnected, made.tagName, made.parentNode]);
    slot.current!.appendChild(made);
    trace("made", "", [made.isConnected, tid(made.parentElement), tid(document.querySelector("[data-testid=made]"))]);
    slot.current!.removeChild(made);
    trace("made", "", [made.isConnected, made.parentNode, document.querySelector("[data-testid=made]")]);
  }, []);
  return (
    <div data-testid="root" className="flex h-full w-full flex-col bg-white" style={{ padding: 16, gap: 8 }}>
      <ul data-testid="list" id="the-list" ref={list} className="flex flex-col" style={{ gap: 4, flexShrink: 0 }}>
        <li data-testid="i0" data-n="0" style={{ width: 100, height: 20, backgroundColor: "rgb(241, 245, 249)" }} />
        <li data-testid="i1" data-n="1" className="on" tabIndex={0} style={{ width: 100, height: 20, backgroundColor: "rgb(14, 165, 233)" }} />
        <li data-testid="i2" data-n="2" style={{ width: 100, height: 20, backgroundColor: "rgb(241, 245, 249)" }}>
          <button data-testid="b" style={{ width: 40, height: 20, backgroundColor: "rgb(225, 29, 72)" }} />
        </li>
      </ul>
      <div data-testid="slot" ref={slot} style={{ width: 100, height: 20, backgroundColor: "rgb(16, 185, 129)", flexShrink: 0 }} />
    </div>
  );
}
