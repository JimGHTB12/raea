// CSS animations and transitions, read once they've ended (their end doesn't depend on when a frame is read): an
// @keyframes animation with fill-mode forwards keeps its last step, one without goes back to the style, Tailwind's
// transition classes ease a background and a transform to the state a click sets.
// proves: css.at-rules.keyframes, css.properties.animation, css.properties.animation-fill-mode, css.properties.transition, css.properties.transition-property
import { useState } from "react";
const CSS = `@keyframes grow { from { transform: none } to { transform: translateX(120px) scale(1.5); opacity: 0.5 } }
@keyframes flash { 50% { background-color: rgb(255, 0, 0) } }
.kept { animation: grow 60ms linear forwards }
.back { animation: flash 60ms }`;
export const steps = [{ wait: 12 }, { click: "toggle" }, { wait: 12 }];
export default function Case() {
  const [on, setOn] = useState(false);
  return (
    <div data-testid="root" className="h-full w-full bg-white p-4 text-slate-900">
      <style>{CSS}</style>
      <button data-testid="toggle" className="bg-slate-200 px-2" onClick={() => setOn(!on)}>toggle</button>
      <div data-testid="kept" className="kept mt-8 h-12 w-12 bg-sky-400" />
      <div data-testid="back" className="back mt-8 h-12 w-12 bg-emerald-400" />
      <div data-testid="eased" className={"mt-8 h-12 w-12 transition duration-75 ease-out " + (on ? "translate-x-24 bg-rose-400" : "bg-amber-300")} />
    </div>
  );
}
