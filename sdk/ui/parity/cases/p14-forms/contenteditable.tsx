// A plain contenteditable (docs/ts-native.feature P14e): click into it, type, Backspace, arrows, Shift+arrows to select
// and type over the selection, End, and across an inline mark; the DOM Chrome's editing makes is what's compared.
// proves: html.global_attributes.contenteditable, api.HTMLElement.isContentEditable, api.Element.beforeinput_event, api.Selection
import { useRef, useState } from "react";
export const steps = [
  { click: "ed", at: [0.04, 0.5] },
  { text: "Say " },
  { key: "End" },
  { text: "!!" },
  { key: "Backspace" },
  { key: "ArrowLeft" },
  { key: "ArrowLeft" },
  { key: "ArrowLeft", mods: ["shift"] },
  { key: "ArrowLeft", mods: ["shift"] },
  { text: "X" },
  { click: "dump" },
];
export default function Case() {
  const ed = useRef<HTMLDivElement>(null);
  const [html, setHtml] = useState("");
  return (
    <div data-testid="root" className="h-full w-full bg-white p-4 text-slate-900">
      <div ref={ed} data-testid="ed" contentEditable suppressContentEditableWarning className="w-80 border border-slate-400 p-2 text-sm outline-none">
        hello <b>bold</b> end
      </div>
      <button data-testid="dump" className="mt-2 border px-2 text-sm" onClick={() => setHtml(ed.current?.innerHTML ?? "")}>
        dump
      </button>
      <pre data-testid="out" className="mt-2 text-xs">
        {html}
      </pre>
    </div>
  );
}
