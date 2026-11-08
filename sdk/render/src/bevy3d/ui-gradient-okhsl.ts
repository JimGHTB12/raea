// bevy_ui's Okhsla and OkhslaLong gradient interpolation (docs/control.feature FC0a), which CSS has no space for. Bevy's
// gradient shader turns each stop to okhsl and mixes there (bevy_render color_operations.wesl mix_okhsl: the hue the
// short way round, or the long way for OkhslaLong, a stop with no saturation taking the other's hue). Here each segment
// between two stops is that mix sampled at SAMPLES points, each an sRGB stop, CSS interpolating between them in Oklab.
import { LinearRgba, Srgba } from "./colors";
import { Okhsla } from "./okhsla";
import { colorCss } from "./ui-val";

const SAMPLES = 24;
/** bevy_render's HUE_GUARD: below this saturation a stop's hue is powerless */
const HUE_GUARD = 0.0001;
const fract = (x: number) => x - Math.floor(x);

type Stop = { color: Srgba | LinearRgba | string; hint?: number };

function okhslOf(who: string, c: Stop["color"]): Okhsla {
  if (c instanceof Srgba || c instanceof LinearRgba) return Okhsla.from(c);
  throw new TypeError(`${who}: an Okhsla gradient's stops are Srgba or LinearRgba colours, got ${JSON.stringify(c)}`);
}

/** mix_okhsl (or mix_okhsl_long): hue in turns, saturation, lightness and alpha mixed */
function mix(a: Okhsla, b: Okhsla, t: number, long: boolean): Okhsla {
  let h = a.hue / 360, g = b.hue / 360;
  if (a.saturation < HUE_GUARD) h = g;
  else if (b.saturation < HUE_GUARD) g = h;
  const d = fract(g - h + 0.5) - 0.5;
  const hue = fract(h + (long ? d + (d > 0 ? -1 : 1) : d) * t);
  const lerp = (x: number, y: number) => x + (y - x) * t;
  return new Okhsla(hue * 360, lerp(a.saturation, b.saturation), lerp(a.lightness, b.lightness), lerp(a.alpha, b.alpha));
}

/** The stops of an Okhsla gradient as CSS's, each segment sampled in okhsl. */
export function okhslStopsCss(who: string, stops: Stop[], at: (s: Stop) => string | undefined, long: boolean): string {
  if (!Array.isArray(stops) || stops.length === 0) throw new TypeError(`${who}: a gradient has stops, got ${JSON.stringify(stops)}`);
  const css = (c: Okhsla) => colorCss(who, "stop", c.to_srgba());
  const out: string[] = [];
  stops.forEach((s, i) => {
    const p = at(s);
    if (p === undefined) throw new TypeError(`${who}: an Okhsla gradient's stops each have a point`);
    if ((s.hint ?? 0.5) !== 0.5) throw new TypeError(`${who}: a hint in an Okhsla gradient is to do; leave it at 0.5`);
    const a = okhslOf(who, s.color);
    out.push(`${css(a)} ${p}`);
    const next = stops[i + 1];
    if (!next) return;
    const q = at(next)!;
    const b = okhslOf(who, next.color);
    for (let k = 1; k < SAMPLES; k++) out.push(`${css(mix(a, b, k / SAMPLES, long))} calc(${p} + (${q} - ${p}) * ${k / SAMPLES})`);
  });
  return out.join(", ");
}
