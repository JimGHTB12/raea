// A field's input events say what changed: inputType and data for text put in, deleteContentBackward for Backspace.
// proves: react-dom.event.onInput, api.InputEvent.inputType, api.InputEvent.data
import { trace } from "parity";
export const steps = [{ click: "f" }, { text: "ab" }, { key: "Backspace" }];
export default function Case() {
  return (
    <div data-testid="root" className="h-full w-full bg-white p-4 text-slate-900">
      <input data-testid="f" className="h-9 w-56 border border-slate-400 px-2 text-sm"
        onInput={(e) => { const n = e.nativeEvent as InputEvent; trace("input", "f", { type: n.inputType, data: n.data, value: e.currentTarget.value }); }} />
    </div>
  );
}
