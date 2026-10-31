// Capture before bubble: onClickCapture on the outer element runs before the inner onClick.
// proves: react-dom.event.onClickCapture, react-dom.event.onClick
import { trace } from "parity";
export const steps = [{ click: "inner" }];
export default function Case() {
  return (
    <div data-testid="root" className="h-full w-full bg-white p-4 text-slate-900">
      <div data-testid="outer" className="w-64 bg-slate-200 p-6" onClickCapture={() => trace("click-capture", "outer")} onClick={() => trace("click", "outer")}>
        <div data-testid="inner" className="h-12 bg-sky-400" onClickCapture={() => trace("click-capture", "inner")} onClick={() => trace("click", "inner")} />
      </div>
    </div>
  );
}
