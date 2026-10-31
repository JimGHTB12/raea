// An img's src changed by a click: the PNG picture, then the poster (solid amber, the same size), then back. Its
// naturalWidth is traced once the new picture has loaded (onLoad).
// proves: html.elements.img.src, api.HTMLImageElement.naturalWidth, api.HTMLImageElement.complete
import { useState } from "react";
import { trace } from "parity";
import { PNG, POSTER } from "./fixtures";
export const steps = [{ click: "swap" }, { wait: 4 }, { click: "swap" }, { wait: 4 }];
export default function Case() {
  const [n, setN] = useState(0);
  return (
    <div data-testid="root" className="flex h-full w-full items-start gap-4 bg-white p-4">
      <img data-testid="img" src={n % 2 ? POSTER : PNG} alt="" className="h-36 w-48" onLoad={(e) => trace("load", "img", (e.target as HTMLImageElement).naturalWidth)} />
      <button data-testid="swap" className="h-8 w-20 bg-slate-800" onClick={() => setN(n + 1)} />
    </div>
  );
}
