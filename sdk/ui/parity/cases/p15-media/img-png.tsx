// An img whose source is PNG: at its natural size (no size given) and stretched to 192x144 by its classes.
// proves: html.elements.img, html.elements.img.src, react-dom.element.img
import { PNG } from "./fixtures";
export const steps = [];
export default function Case() {
  return (
    <div data-testid="root" className="flex h-full w-full items-start gap-4 bg-white p-4">
      <img data-testid="natural" src={PNG} alt="" />
      <img data-testid="sized" src={PNG} alt="" className="h-36 w-48" />
    </div>
  );
}
