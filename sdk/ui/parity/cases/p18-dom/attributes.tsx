// P18: attributes, tagName, textContent, id, className, classList, dataset and the style object on host elements,
// read as React DOM sets them from props, and written from JS (kept across React's later renders, as React DOM keeps
// what it doesn't own).
// proves: api.Element.getAttribute, api.Element.setAttribute, api.Element.removeAttribute, api.Element.hasAttribute, api.Element.toggleAttribute, api.Element.getAttributeNames, api.Element.tagName, api.Element.localName, api.Node.nodeName, api.Node.textContent, api.Element.id, api.Element.className, api.Element.classList, api.HTMLElement.dataset, api.HTMLElement.style, api.CSSStyleDeclaration.setProperty, api.CSSStyleDeclaration.getPropertyValue, api.CSSStyleDeclaration.removeProperty
// tier: quick
import { useEffect, useRef, useState } from "react";
import { trace } from "parity";

export const steps = [{ click: "flip" }];

export default function Case() {
  const box = useRef<HTMLDivElement>(null);
  const label = useRef<HTMLSpanElement>(null);
  const [on, setOn] = useState(false);
  useEffect(() => {
    const b = box.current!;
    trace("read", "box", [b.tagName, b.localName, b.nodeName, b.id, b.className, b.getAttribute("data-kind"), b.getAttribute("aria-hidden"), b.getAttribute("tabindex"), b.hasAttribute("title"), b.getAttribute("role")]);
    trace("dataset", "box", [b.dataset.kind, b.dataset.longName, "kind" in b.dataset]);
    trace("text", "label", [label.current!.textContent, label.current!.tagName]);
    b.setAttribute("data-made", "yes");
    b.classList.add("extra");
    b.classList.remove("b");
    trace("classes", "box", [b.className, b.classList.contains("a"), b.classList.length, b.classList.toggle("t"), b.className]);
    b.toggleAttribute("data-flag");
    trace("toggle", "box", [b.hasAttribute("data-flag"), b.getAttribute("data-flag"), b.toggleAttribute("data-flag"), b.hasAttribute("data-flag")]);
    b.removeAttribute("title");
    trace("names", "box", b.getAttributeNames().sort());
    b.style.setProperty("--gap", "4px");
    b.style.setProperty("border-top-left-radius", "6px");
    trace("style", "box", [b.style.getPropertyValue("--gap"), b.style.getPropertyValue("width"), b.style.width, b.style.getPropertyValue("border-top-left-radius"), b.style.removeProperty("--gap"), b.style.getPropertyValue("--gap")]);
  }, []);
  return (
    <div data-testid="root" className="flex h-full w-full flex-col bg-white" style={{ padding: 16, gap: 8 }}>
      <div
        data-testid="box"
        ref={box}
        id="the-box"
        className={on ? "a b on" : "a b"}
        data-kind="card"
        data-long-name="x"
        aria-hidden={false}
        tabIndex={-1}
        title="tip"
        role="group"
        style={{ width: 160, height: 60, backgroundColor: on ? "rgb(14, 165, 233)" : "rgb(225, 29, 72)", flexShrink: 0 }}
      />
      {/* its text in the background's colour: how text is drawn is P13's */}
      <span data-testid="label" ref={label} style={{ fontSize: 16, height: 24, color: "rgb(255, 255, 255)" }}>
        hello
      </span>
      <div
        data-testid="flip"
        onClick={() => {
          setOn(true);
          // after React's render: what JS set is still there, what React set changed
          requestAnimationFrame(() => {
            const b = box.current!;
            trace("after", "box", [b.className, b.getAttribute("data-made"), b.style.width, b.style.getPropertyValue("border-top-left-radius")]);
          });
        }}
        style={{ width: 80, height: 32, backgroundColor: "rgb(30, 41, 59)", flexShrink: 0 }}
      />
    </div>
  );
}
