// Images and graphics: an img named by alt, an svg with role=img and aria-label, an svg named by its <title>, a
// decorative svg (aria-hidden), and an svg with nothing to name it.
// proves: aria.role.img, aria.html-aam.el-img, aria.html-aam.att-alt, svg.elements.title
import { PNG } from "../p15-media/fixtures";
export const steps = [];
export default function Case() {
  return (
    <div data-testid="root" className="h-full w-full bg-white p-4">
      <div className="flex items-start gap-2 opacity-0">
        <img data-testid="img" src={PNG} alt="Logo" className="h-8 w-8" />
        <svg data-testid="svg-label" role="img" aria-label="Star" width={32} height={32}><circle cx={16} cy={16} r={10} /></svg>
        <svg data-testid="svg-title" width={32} height={32}><title>Moon</title><circle cx={16} cy={16} r={10} /></svg>
        <svg data-testid="svg-hidden" aria-hidden="true" width={32} height={32}><circle cx={16} cy={16} r={10} /></svg>
        <svg data-testid="svg-bare" width={32} height={32}><circle cx={16} cy={16} r={10} /></svg>
      </div>
    </div>
  );
}
