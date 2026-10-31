// Transitions on lengths and the text colour, read once they've ended: a width, a max-height (an accordion's), an
// inset and a colour eased to what a click sets; a @keyframes animation of a width kept at its end.
// proves: css.properties.transition, css.properties.transition-property, css.at-rules.keyframes
import { useState } from "react";
const CSS = `@keyframes fill { from { width: 0px } to { width: 240px } }
.bar { animation: fill 60ms ease-out forwards }`;
export const steps = [{ wait: 12 }, { click: "toggle" }, { wait: 12 }];
export default function Case() {
  const [on, setOn] = useState(false);
  return (
    <div data-testid="root" className="relative h-full w-full bg-white p-4 text-slate-900">
      <style>{CSS}</style>
      <button data-testid="toggle" className="bg-slate-200 px-2" onClick={() => setOn(!on)}>toggle</button>
      <div data-testid="wide" className="mt-4 h-8 bg-sky-400" style={{ width: on ? 300 : 100, transition: "width 60ms linear" }} />
      <div data-testid="fold" className="mt-4 overflow-hidden bg-amber-200" style={{ maxHeight: on ? 120 : 24, transition: "max-height 60ms" }}>
        <div className="h-32" />
      </div>
      <div data-testid="moved" className="absolute h-8 w-8 bg-rose-400" style={{ left: on ? 400 : 200, top: 40, transition: "left 60ms ease-in" }} />
      <p data-testid="ink" className="mt-4" style={{ color: on ? "rgb(220, 38, 38)" : "rgb(15, 23, 42)", transition: "color 60ms" }}>colour</p>
      <div data-testid="bar" className="bar mt-4 h-4 bg-emerald-500" />
    </div>
  );
}
