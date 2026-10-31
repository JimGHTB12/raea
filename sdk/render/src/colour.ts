// Colours as the 3D renderer takes them (#rrggbb). 3d.tsx exports toHex for the SDK's other 3D modules.

const NAMED: Record<string, string> = {
  white: "#ffffff", black: "#000000", red: "#ff0000", green: "#008000", blue: "#0000ff", yellow: "#ffff00", orange: "#ffa500",
  purple: "#800080", gray: "#808080", grey: "#808080", pink: "#ffc0cb", cyan: "#00ffff", magenta: "#ff00ff", gold: "#ffd700", silver: "#c0c0c0",
};

/** A colour as the 3D renderer takes it (#rrggbb): hex, rgb()/rgba() or a common name; CSS variables throw. */
export function toHex(owner: string, k: string, v: unknown): string {
  if (typeof v === "string") {
    const c = v.trim().toLowerCase();
    if (/^#[0-9a-f]{6}$/.test(c)) return c;
    if (/^#[0-9a-f]{3}$/.test(c)) return "#" + [...c.slice(1)].map((x) => x + x).join("");
    const m = c.match(/^rgba?\(\s*([\d.]+)\s*,\s*([\d.]+)\s*,\s*([\d.]+)/);
    if (m) return "#" + m.slice(1, 4).map((x) => Math.max(0, Math.min(255, Math.round(Number(x)))).toString(16).padStart(2, "0")).join("");
    if (NAMED[c]) return NAMED[c];
  }
  throw new TypeError(`${owner}: ${k} must be a colour like "#ff8800" (the 3D renderer reads no CSS variables), got ${JSON.stringify(v)}`);
}
