// An svg React updates: a click grows a circle, recolours a gradient's stop and toggles a path; the drawing changes in
// the frame the click landed in.
// proves: svg.elements.circle, svg.elements.stop, react-dom.element.svg
import { useState } from "react";
export const steps = [{ click: "go" }, { click: "go" }];
export default function Case() {
  const [n, setN] = useState(0);
  return (
    <div data-testid="root" className="flex h-full w-full items-start gap-4 bg-white p-4">
      <svg data-testid="svg" width={240} height={160}>
        <defs>
          <linearGradient id="up-g"><stop offset="0" stopColor={n % 2 ? "#16a34a" : "#e11d48"} /><stop offset="1" stopColor="#2563eb" /></linearGradient>
        </defs>
        <rect width={240} height={60} fill="url(#up-g)" />
        <circle cx={60} cy={110} r={20 + n * 12} fill="#f59e0b" />
        {n === 1 && <path d="M140 80 L230 150" stroke="#111827" strokeWidth={8} />}
      </svg>
      <button data-testid="go" className="h-8 w-20 bg-slate-800" onClick={() => setN(n + 1)} />
    </div>
  );
}
