// A Bevy 3D view as an element (view3d: a camera of its own rendering into the element's box, bevy_ui's
// ViewportNode): two views side by side, each its camera's clear colour, with an empty scene. Chrome has no Bevy, so
// the oracle is what the views must show, drawn as divs.
import { side } from "parity";
export const steps = [];
const View = (side === "bevy" ? "view3d" : "div") as any;
export default function Case() {
  const clear = (c: string) => (side === "bevy" ? { clearColor: c } : { style: { backgroundColor: c } });
  return (
    <div data-testid="root" className="flex h-full w-full items-start gap-4 bg-white p-4">
      <View data-testid="a" className="h-[150px] w-[200px] shrink-0" {...clear("#1e293b")} />
      <View data-testid="b" className="h-[100px] w-[120px] shrink-0" {...clear("#0891b2")} />
    </div>
  );
}
