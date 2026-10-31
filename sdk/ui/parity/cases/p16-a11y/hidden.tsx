// What the tree leaves out: display none, the hidden attribute, aria-hidden (and what's inside it), visibility
// hidden, role=presentation and role=none (their content stays, as text of their parent), an img with alt="".
// proves: aria.state.aria-hidden, aria.role.presentation, aria.role.none, aria.html-aam.att-hidden, aria.html-aam.el-img-empty-alt
import { PNG } from "../p15-media/fixtures";
export const steps = [];
const f = "block h-6 w-40";
export default function Case() {
  return (
    <div data-testid="root" className="h-full w-full bg-white p-4">
      <div className="opacity-0">
        <button data-testid="shown" className={f}>Shown</button>
        <button data-testid="display-none" className={`${f} hidden`}>Gone</button>
        <button data-testid="hidden-attr" hidden className={f}>Gone</button>
        <div data-testid="aria-hidden" aria-hidden="true" className="block w-40">
          <button data-testid="inside-hidden" className={f}>Inside</button>
        </div>
        <button data-testid="invisible" className={`${f} invisible`}>Invisible</button>
        <div data-testid="presentation" role="presentation" className={f}>kept text</div>
        <div data-testid="none" role="none" className={f} />
        <img data-testid="decorative" src={PNG} alt="" className="h-6 w-8" />
      </div>
    </div>
  );
}
