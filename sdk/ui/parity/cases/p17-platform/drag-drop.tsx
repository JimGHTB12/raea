// HTML drag and drop with the mouse: a draggable chip dragged onto a zone, with dragstart (dataTransfer.setData),
// dragenter and dragover (preventDefault to allow the drop), drop (getData), and dragend's dropEffect.
// proves: api.DataTransfer, api.DataTransfer.setData, api.DataTransfer.getData, api.DataTransfer.types, api.DataTransfer.effectAllowed, api.DataTransfer.dropEffect
import { useState } from "react";
import { trace } from "parity";
export const steps = [{ down: "chip" }, { move: [80, 40] }, { move: [200, 60] }, { move: "zone" }, { up: "zone" }, { wait: 4 }];
export default function Case() {
  const [dropped, setDropped] = useState("");
  return (
    <div data-testid="root" className="flex h-full w-full items-start gap-8 bg-white p-4">
      <div
        data-testid="chip"
        draggable
        className="h-10 w-20 bg-indigo-400"
        onDragStart={(e) => {
          e.dataTransfer.setData("text/plain", "chip-1");
          e.dataTransfer.effectAllowed = "move";
          trace("dragstart", "chip", [e.dataTransfer.types]);
        }}
        onDragEnd={(e) => trace("dragend", "chip", e.dataTransfer.dropEffect)}
      />
      <div
        data-testid="zone"
        data-value={dropped}
        className={dropped ? "h-32 w-48 bg-emerald-300" : "h-32 w-48 bg-slate-200"}
        onDragEnter={() => trace("dragenter", "zone")}
        onDragOver={(e) => {
          e.preventDefault();
          e.dataTransfer.dropEffect = "move";
        }}
        onDrop={(e) => {
          e.preventDefault();
          const v = e.dataTransfer.getData("text/plain");
          trace("drop", "zone", v);
          setDropped(v);
        }}
      />
    </div>
  );
}
