// An img's intrinsic size: none given (96x72), only a width (the height follows the picture's aspect), only a height,
// the width and height attributes (as React sets them), and max-width shrinking it in a narrow parent.
// proves: html.elements.img, html.elements.img.width, html.elements.img.height, api.HTMLImageElement.naturalWidth
import { PNG } from "./fixtures";
export const steps = [];
export default function Case() {
  return (
    <div data-testid="root" className="flex h-full w-full items-start gap-4 bg-white p-4">
      <img data-testid="none" src={PNG} alt="" />
      <img data-testid="w-only" src={PNG} alt="" className="w-[144px]" />
      <img data-testid="h-only" src={PNG} alt="" className="h-[36px]" />
      <img data-testid="attrs" src={PNG} alt="" width={48} height={36} />
      <div data-testid="narrow" className="w-[60px] shrink-0">
        <img data-testid="max-w" src={PNG} alt="" className="max-w-full" />
      </div>
    </div>
  );
}
