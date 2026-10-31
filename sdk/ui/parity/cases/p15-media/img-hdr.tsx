// An img whose source is Radiance HDR, a format Bevy decodes and Chrome doesn't: the oracle shows the picture the HDR
// file holds as a PNG (its shared exponent rounds a dark channel beside a bright one), Bevy the Radiance HDR. At its natural size (no size given) and stretched to 192x144 by its classes.
// proves: html.elements.img, html.elements.img.src, react-dom.element.img
import { side } from "parity";
import { HDR, HDR_PNG } from "./fixtures";
const src = side === "bevy" ? HDR : HDR_PNG;
export const steps = [];
export default function Case() {
  return (
    <div data-testid="root" className="flex h-full w-full items-start gap-4 bg-white p-4">
      <img data-testid="natural" src={src} alt="" />
      <img data-testid="sized" src={src} alt="" className="h-36 w-48" />
    </div>
  );
}
