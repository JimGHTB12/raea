// @font-face in a <style>: a family registered from a data URL (woff2), used by style.fontFamily, by font-[...] and
// inherited from a parent; a 700 face picked by font-bold; a family that isn't there falls back to the page's face.
// proves: css.at-rules.font-face, css.properties.font-family, css.properties.font-weight
import { OSWALD_400, OSWALD_700 } from "./fonts";
export const steps = [];
const css = `@font-face { font-family: "Brand"; font-weight: 400; src: url(${OSWALD_400}) format("woff2"); }
@font-face { font-family: "Brand"; font-weight: 700; src: url(${OSWALD_700}) format("woff2"); }`;
export default function Case() {
  return (
    <div data-testid="root" className="h-full w-full bg-white p-4 text-slate-900">
      <style>{css}</style>
      <p data-testid="styled" style={{ fontFamily: "Brand, sans-serif" }}><span data-testid="s1">Styled with Brand</span></p>
      <p data-testid="arb" className="font-[Brand] text-2xl"><span data-testid="s2">Arbitrary class</span></p>
      <div style={{ fontFamily: "Brand" }}>
        <p data-testid="inherit"><span data-testid="s3">Inherited</span> <span data-testid="s4" className="font-bold">bold face</span></p>
      </div>
      <p data-testid="missing" style={{ fontFamily: "Nowhere, Brand-Not" }}><span data-testid="s5">Fallback face</span></p>
    </div>
  );
}
