// text-transform: uppercase, lowercase and capitalize change the drawn text (and so its width), inherited by the
// text below; the text itself is unchanged (textContent reads it as written).
// proves: css.properties.text-transform
import { useRef, useEffect } from "react";
import { trace } from "parity";
export const steps = [];
export default function Case() {
  const p = useRef<HTMLParagraphElement>(null);
  useEffect(() => trace("text", "up", p.current?.textContent), []);
  return (
    <div data-testid="root" className="h-full w-full bg-white p-4 text-slate-900">
      <p data-testid="up" ref={p} className="uppercase"><span data-testid="up-s">Mixed Case words</span></p>
      <p data-testid="low" className="lowercase"><span data-testid="low-s">LOUD Words</span></p>
      <p data-testid="cap" style={{ textTransform: "capitalize" }}><span data-testid="cap-s">each word here</span></p>
      <div className="uppercase"><p data-testid="inh"><span data-testid="inh-s">inherited too</span></p></div>
    </div>
  );
}
