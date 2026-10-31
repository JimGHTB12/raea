// An img whose source is JPEG (quality 95, 4, a format Bevy decodes and Chrome doesn't: the oracle shows the same picture as a
// PNG, Bevy the JPEG (quality 95, 4. At its natural size (no size given) and stretched to 192x144 by its classes.
// proves: html.elements.img, html.elements.img.src, react-dom.element.img
import { side } from "parity";
import { PNG, JPG } from "./fixtures";
const src = side === "bevy" ? JPG : PNG;
export const steps = [];
export default function Case() {
  return (
    <div data-testid="root" className="flex h-full w-full items-start gap-4 bg-white p-4">
      <img data-testid="natural" src={src} alt="" />
      <img data-testid="sized" src={src} alt="" className="h-36 w-48" />
    </div>
  );
}
