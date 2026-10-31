// An img whose source is OpenEXR (32-bit float), a format Bevy decodes and Chrome doesn't: the oracle shows the same picture as a
// PNG, Bevy the OpenEXR (32-bit float). At its natural size (no size given) and stretched to 192x144 by its classes.
// proves: html.elements.img, html.elements.img.src, react-dom.element.img
import { side } from "parity";
import { PNG, EXR } from "./fixtures";
const src = side === "bevy" ? EXR : PNG;
export const steps = [];
export default function Case() {
  return (
    <div data-testid="root" className="flex h-full w-full items-start gap-4 bg-white p-4">
      <img data-testid="natural" src={src} alt="" />
      <img data-testid="sized" src={src} alt="" className="h-36 w-48" />
    </div>
  );
}
