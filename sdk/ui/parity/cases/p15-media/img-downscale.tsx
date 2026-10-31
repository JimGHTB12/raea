// A picture drawn below its size (B4b): a 192x144 one-pixel checker at 0.67x, 0.5x and 0.4x, as an img and as a
// video's frame. Chrome minifies through mip levels, so a checker drawn small comes out an even blend of its two
// colours; sampled at full size it aliases into stripes and moiré. The img's last box is image-rendering: pixelated,
// which Chrome samples nearest with no mips.
// proves: html.elements.img, html.elements.video, css.properties.image-rendering
import { CHECKER, CHECKER_VIDEO } from "./checker";
export const steps = [{ wait: 10 }];
const SCALES = [0.67, 0.5, 0.4];
export default function Case() {
  return (
    <div data-testid="root" className="flex h-full w-full flex-col items-start gap-4 bg-white p-4">
      <div className="flex items-start gap-4">
        {SCALES.map((s) => (
          <img key={s} data-testid={`img-${s}`} src={CHECKER} alt="" style={{ width: Math.round(192 * s), height: Math.round(144 * s) }} />
        ))}
        <img data-testid="img-pixelated" src={CHECKER} alt="" style={{ width: 129, height: 96, imageRendering: "pixelated" }} />
      </div>
      <div className="flex items-start gap-4">
        {SCALES.map((s) => (
          <video key={s} data-testid={`video-${s}`} src={CHECKER_VIDEO} muted style={{ width: Math.round(192 * s), height: Math.round(144 * s) }} />
        ))}
      </div>
    </div>
  );
}
