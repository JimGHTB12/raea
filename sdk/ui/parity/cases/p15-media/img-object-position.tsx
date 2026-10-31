// object-position: the 96x72 picture contained in 200x80 boxes (free space across) and covering 60x150 boxes (cropped
// across), placed left, right, centre and by style (25% 75%, 10px 20px), and at its own size (object-none) in a
// 150x150 box at each corner.
// proves: css.properties.object-position, css.properties.object-position.left, css.properties.object-position.right, css.properties.object-position.top, css.properties.object-position.bottom, css.properties.object-position.center, tw.utility.object-left-top, tw.utility.object-right-bottom
import { PNG } from "./fixtures";
export const steps = [];
const wide = "h-[80px] w-[200px] shrink-0 bg-slate-300 object-contain";
const tall = "h-[150px] w-[60px] shrink-0 bg-slate-300 object-cover";
const own = "h-[150px] w-[150px] shrink-0 bg-slate-300 object-none";
export default function Case() {
  return (
    <div data-testid="root" className="flex h-full w-full flex-wrap content-start items-start gap-3 bg-white p-4">
      <img data-testid="contain-left" src={PNG} alt="" className={`${wide} object-left`} />
      <img data-testid="contain-right" src={PNG} alt="" className={`${wide} object-right`} />
      <img data-testid="contain-pct" src={PNG} alt="" className={wide} style={{ objectPosition: "25% 75%" }} />
      <img data-testid="cover-left" src={PNG} alt="" className={`${tall} object-left`} />
      <img data-testid="cover-right" src={PNG} alt="" className={`${tall} object-right`} />
      <img data-testid="cover-px" src={PNG} alt="" className={tall} style={{ objectPosition: "-10px 0px" }} />
      <img data-testid="none-lt" src={PNG} alt="" className={`${own} object-left-top`} />
      <img data-testid="none-rb" src={PNG} alt="" className={`${own} object-right-bottom`} />
      <img data-testid="none-px" src={PNG} alt="" className={own} style={{ objectPosition: "10px 20px" }} />
    </div>
  );
}
