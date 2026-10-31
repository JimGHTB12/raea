// Accessible names as Chrome computes them: a button's text (its span's too), aria-label over the text,
// aria-labelledby (two elements, one named by its own aria-label), an image's alt inside a button, a title when there's
// nothing else, a link's text, a role=button span's text, an img's alt, and alt="" making the img presentational.
// proves: aria.property.aria-label, aria.property.aria-labelledby, aria.role.button, aria.role.link, aria.role.img, aria.html-aam.el-button, aria.html-aam.el-a, aria.html-aam.el-img, aria.html-aam.el-img-empty-alt, aria.html-aam.att-alt, aria.html-aam.att-title
import { PNG } from "../p15-media/fixtures";
export const steps = [];
const b = "mr-2 inline-block h-6 w-20";
export default function Case() {
  return (
    <div data-testid="root" className="h-full w-full bg-white p-4">
      <div className="flex flex-wrap items-start gap-2 opacity-0">
        <button data-testid="text" className={b}>Save <span>now</span></button>
        <button data-testid="label" aria-label="Close" className={b}>X</button>
        <button data-testid="labelledby" aria-labelledby="nm-one nm-two" className={b} />
        <span id="nm-one" className={b}>one</span>
        <span id="nm-two" aria-label="two" className={b}>zz</span>
        <button data-testid="img-alt" className={b}><img src={PNG} alt="delete" className="h-4 w-4" /></button>
        <button data-testid="title" title="Settings" className={b} />
        <a data-testid="link" href="#next" className={b}>Next page</a>
        <span data-testid="span-button" role="button" tabIndex={0} className={b}>Go</span>
        <img data-testid="img" src={PNG} alt="A picture" className="h-6 w-8" />
        <img data-testid="img-empty" src={PNG} alt="" className="h-6 w-8" />
      </div>
    </div>
  );
}
