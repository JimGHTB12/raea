// object-fit on a 96x72 picture in 120x120 boxes: fill (stretched), contain (letterboxed), cover (cropped), none (its
// own size, centred, clipped to the box) and scale-down (contain here, as the picture is smaller than the box: its own
// size). Each box has a slate background, so the letterbox shows.
// proves: css.properties.object-fit, css.properties.object-fit.fill, css.properties.object-fit.contain, css.properties.object-fit.cover, css.properties.object-fit.none, css.properties.object-fit.scale-down, tw.utility.object-fill, tw.utility.object-contain, tw.utility.object-cover, tw.utility.object-none, tw.utility.object-scale-down
import { PNG } from "./fixtures";
export const steps = [];
const box = "h-[120px] w-[120px] shrink-0 bg-slate-300";
export default function Case() {
  return (
    <div data-testid="root" className="flex h-full w-full flex-wrap items-start gap-4 bg-white p-4">
      <img data-testid="fill" src={PNG} alt="" className={`${box} object-fill`} />
      <img data-testid="contain" src={PNG} alt="" className={`${box} object-contain`} />
      <img data-testid="cover" src={PNG} alt="" className={`${box} object-cover`} />
      <img data-testid="none" src={PNG} alt="" className={`${box} object-none`} />
      <img data-testid="scale-down" src={PNG} alt="" className={`${box} object-scale-down`} />
      <img data-testid="wide-contain" src={PNG} alt="" className="h-[60px] w-[200px] shrink-0 bg-slate-300 object-contain" />
      <img data-testid="tall-cover" src={PNG} alt="" className="h-[150px] w-[50px] shrink-0 bg-slate-300 object-cover" />
    </div>
  );
}
