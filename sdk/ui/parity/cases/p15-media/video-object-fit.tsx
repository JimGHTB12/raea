// object-fit on a video's picture (its poster here, 96x72) in 160x80 boxes: contain (letterboxed: the box's slate
// background either side) and cover (cropped), and the default (contain, for a video).
// proves: css.properties.object-fit.contain, css.properties.object-fit.cover, html.elements.video
import { POSTER, VIDEO } from "./fixtures";
export const steps = [{ wait: 10 }];
const box = "h-[80px] w-[160px] shrink-0 bg-slate-300";
export default function Case() {
  return (
    <div data-testid="root" className="flex h-full w-full items-start gap-4 bg-white p-4">
      <video data-testid="default" src={VIDEO} poster={POSTER} muted className={box} />
      <video data-testid="contain" src={VIDEO} poster={POSTER} muted className={`${box} object-contain`} />
      <video data-testid="cover" src={VIDEO} poster={POSTER} muted className={`${box} object-cover`} />
    </div>
  );
}
