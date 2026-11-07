// bevy_ui's grid fields (grid_template_rows and _columns of RepeatedGridTracks, grid_auto_rows and _columns of
// GridTracks, grid_auto_flow, grid_row and grid_column GridPlacements) as CSS's, which taffy lays out for Blitz as it
// does for Bevy.

/** bevy_ui's MinTrackSizingFunction. @bevy bevy_ui::MinTrackSizingFunction */
export type MinTrackSizingFunction =
  | "Auto" | "MinContent" | "MaxContent"
  | { Px: number } | { Percent: number } | { Rem: number } | { Vw: number } | { Vh: number } | { VMin: number } | { VMax: number };
/** bevy_ui's MaxTrackSizingFunction: a MinTrackSizingFunction, a Fraction of the free space, or a fit-content limit.
 *  @bevy bevy_ui::MaxTrackSizingFunction */
export type MaxTrackSizingFunction = MinTrackSizingFunction | { Fraction: number } | { FitContentPx: number } | { FitContentPercent: number };

/** bevy_ui's GridTrack: a track sized between its min and max sizing functions. @bevy bevy_ui::GridTrack */
export interface GridTrack { min_sizing_function: MinTrackSizingFunction; max_sizing_function: MaxTrackSizingFunction }
/** bevy_ui's GridTrackRepetition: a count, or as many as fit (AutoFill), the empty ones collapsed (AutoFit).
 *  @bevy bevy_ui::GridTrackRepetition */
export type GridTrackRepetition = number | "AutoFill" | "AutoFit";
/** bevy_ui's RepeatedGridTrack: tracks repeated. @bevy bevy_ui::RepeatedGridTrack */
export interface RepeatedGridTrack { repetition: GridTrackRepetition; tracks: GridTrack[] }

const both = (f: MinTrackSizingFunction): GridTrack => ({ min_sizing_function: f, max_sizing_function: f });
const track = {
  px: (v: number) => both({ Px: v }),
  percent: (v: number) => both({ Percent: v }),
  rem: (v: number) => both({ Rem: v }),
  vw: (v: number) => both({ Vw: v }),
  vh: (v: number) => both({ Vh: v }),
  vmin: (v: number) => both({ VMin: v }),
  vmax: (v: number) => both({ VMax: v }),
  /** CSS's fr: at least its content (Auto), a share of the free space */
  fr: (v: number): GridTrack => ({ min_sizing_function: "Auto", max_sizing_function: { Fraction: v } }),
  /** a share of the free space from 0 (Bevy's flex: minmax(0, fr)) */
  flex: (v: number): GridTrack => ({ min_sizing_function: { Px: 0 }, max_sizing_function: { Fraction: v } }),
  auto: () => both("Auto"),
  min_content: () => both("MinContent"),
  max_content: () => both("MaxContent"),
  fit_content_px: (v: number): GridTrack => ({ min_sizing_function: "Auto", max_sizing_function: { FitContentPx: v } }),
  fit_content_percent: (v: number): GridTrack => ({ min_sizing_function: "Auto", max_sizing_function: { FitContentPercent: v } }),
  minmax: (min: MinTrackSizingFunction, max: MaxTrackSizingFunction): GridTrack => ({ min_sizing_function: min, max_sizing_function: max }),
};
export const GridTrack = track;

type Sized = Exclude<keyof typeof track, "auto" | "min_content" | "max_content" | "minmax">;
const repeated = (k: Sized) => (repetition: GridTrackRepetition, v: number): RepeatedGridTrack => ({ repetition, tracks: [track[k](v)] });
/** bevy_ui's RepeatedGridTrack constructors: each GridTrack's, repeated `repetition` times. */
export const RepeatedGridTrack = {
  px: repeated("px"), percent: repeated("percent"), rem: repeated("rem"), vw: repeated("vw"), vh: repeated("vh"), vmin: repeated("vmin"), vmax: repeated("vmax"),
  fr: repeated("fr"), flex: repeated("flex"), fit_content_px: repeated("fit_content_px"), fit_content_percent: repeated("fit_content_percent"),
  auto: (repetition: GridTrackRepetition): RepeatedGridTrack => ({ repetition, tracks: [track.auto()] }),
  min_content: (repetition: GridTrackRepetition): RepeatedGridTrack => ({ repetition, tracks: [track.min_content()] }),
  max_content: (repetition: GridTrackRepetition): RepeatedGridTrack => ({ repetition, tracks: [track.max_content()] }),
  minmax: (repetition: GridTrackRepetition, min: MinTrackSizingFunction, max: MaxTrackSizingFunction): RepeatedGridTrack => ({ repetition, tracks: [track.minmax(min, max)] }),
  repeat_many: (repetition: GridTrackRepetition, tracks: GridTrack[]): RepeatedGridTrack => ({ repetition, tracks }),
};

/** bevy_ui's GridPlacement: a start line, an end line (1-based, negative from the end) and a span (1 by default).
 *  @bevy bevy_ui::GridPlacement */
export interface GridPlacement { start?: number; span?: number; end?: number }
export const GridPlacement = {
  DEFAULT: {} as GridPlacement,
  auto: (): GridPlacement => ({}),
  start: (start: number): GridPlacement => ({ start }),
  end: (end: number): GridPlacement => ({ end }),
  span: (span: number): GridPlacement => ({ span }),
  start_span: (start: number, span: number): GridPlacement => ({ start, span }),
  start_end: (start: number, end: number): GridPlacement => ({ start, end }),
  end_span: (end: number, span: number): GridPlacement => ({ end, span }),
};

/** bevy_ui's GridAutoFlow. @bevy bevy_ui::GridAutoFlow */
export type GridAutoFlow = "Row" | "Column" | "RowDense" | "ColumnDense";
const FLOW: Record<GridAutoFlow, string> = { Row: "row", Column: "column", RowDense: "row dense", ColumnDense: "column dense" };

const num = (who: string, k: string, v: unknown, ok: (n: number) => boolean = Number.isFinite) => {
  if (typeof v !== "number" || !ok(v)) throw new TypeError(`${who}: ${k} is a number, got ${JSON.stringify(v)}`);
  return v;
};

function sizing(who: string, f: MaxTrackSizingFunction): string {
  if (f === "Auto") return "auto";
  if (f === "MinContent") return "min-content";
  if (f === "MaxContent") return "max-content";
  const [k, v] = typeof f === "object" && f ? (Object.entries(f)[0] ?? []) : [];
  const n = () => num(who, `a track's ${k}`, v);
  switch (k) {
    case "Px": return `${n()}px`;
    case "Percent": return `${n()}%`;
    // (bevy_text's 20 px rem)
    case "Rem": return `${n() * 20}px`;
    case "Vw": return `${n()}vw`;
    case "Vh": return `${n()}vh`;
    case "VMin": return `${n()}vmin`;
    case "VMax": return `${n()}vmax`;
    case "Fraction": return `${num(who, "a track's Fraction", v, (x) => Number.isFinite(x) && x >= 0)}fr`;
    default: throw new TypeError(`${who}: a track's sizing is Auto, MinContent, MaxContent or { Px | Percent | Rem | Vw | Vh | VMin | VMax | Fraction | FitContentPx | FitContentPercent }, got ${JSON.stringify(f)}`);
  }
}

/** One GridTrack as CSS's. */
export function trackCss(who: string, t: GridTrack): string {
  if (!t || typeof t !== "object" || !("min_sizing_function" in t)) throw new TypeError(`${who}: a track is a GridTrack (GridTrack.px(10)), got ${JSON.stringify(t)}`);
  const { min_sizing_function: lo, max_sizing_function: hi } = t;
  if (typeof hi === "object" && hi && "FitContentPx" in hi) return lo === "Auto" ? `fit-content(${num(who, "FitContentPx", hi.FitContentPx)}px)` : `minmax(${sizing(who, lo)}, fit-content(${hi.FitContentPx}px))`;
  if (typeof hi === "object" && hi && "FitContentPercent" in hi) return lo === "Auto" ? `fit-content(${num(who, "FitContentPercent", hi.FitContentPercent)}%)` : `minmax(${sizing(who, lo)}, fit-content(${hi.FitContentPercent}%))`;
  if (typeof lo === "object" && lo && "Fraction" in (lo as object)) throw new TypeError(`${who}: a track's min_sizing_function can't be a Fraction`);
  const [a, b] = [sizing(who, lo), sizing(who, hi)];
  // CSS's fr alone is minmax(auto, fr), as Bevy's fr
  if (a === b || (lo === "Auto" && b.endsWith("fr"))) return b;
  return `minmax(${a}, ${b})`;
}

/** grid_template_rows or _columns (RepeatedGridTracks, or GridTracks once each) as CSS's track list. */
export function templateCss(who: string, k: string, ts: (RepeatedGridTrack | GridTrack)[]): string {
  if (!Array.isArray(ts)) throw new TypeError(`${who}: ${k} is a list of RepeatedGridTrack, got ${JSON.stringify(ts)}`);
  return ts.map((t) => {
    if (!("repetition" in t)) return trackCss(who, t);
    const r = t.repetition;
    const list = t.tracks.map((x) => trackCss(who, x)).join(" ");
    if (r === "AutoFill" || r === "AutoFit") return `repeat(${r === "AutoFill" ? "auto-fill" : "auto-fit"}, ${list})`;
    num(who, `${k}'s repetition`, r, (n) => Number.isInteger(n) && n >= 1);
    return r === 1 ? list : `repeat(${r}, ${list})`;
  }).join(" ");
}

/** grid_row or grid_column as CSS's. */
export function placementCss(who: string, k: string, p: GridPlacement): string {
  if (!p || typeof p !== "object") throw new TypeError(`${who}: ${k} is a GridPlacement (GridPlacement.span(2)), got ${JSON.stringify(p)}`);
  const line = (n: number | undefined, w: string) => (n === undefined ? undefined : `${num(who, `${k}.${w}`, n, (x) => Number.isInteger(x) && x !== 0)}`);
  const span = p.span === undefined ? undefined : `span ${num(who, `${k}.span`, p.span, (x) => Number.isInteger(x) && x >= 1)}`;
  const [s, e] = [line(p.start, "start"), line(p.end, "end")];
  if (s !== undefined && e !== undefined && span !== undefined) throw new TypeError(`${who}: ${k} has at most two of start, end and span`);
  if (s !== undefined) return `${s} / ${e ?? span ?? "auto"}`;
  if (e !== undefined) return `${span ?? "auto"} / ${e}`;
  return span ?? "auto";
}

export function flowCss(who: string, f: GridAutoFlow): string {
  const s = FLOW[f];
  if (!s) throw new TypeError(`${who}: grid_auto_flow is one of ${Object.keys(FLOW).join(", ")}, got ${JSON.stringify(f)}`);
  return s;
}

/** A Node's grid fields into its CSS. */
export function gridCss(who: string, p: { grid_template_rows?: (RepeatedGridTrack | GridTrack)[]; grid_template_columns?: (RepeatedGridTrack | GridTrack)[]; grid_auto_rows?: GridTrack[]; grid_auto_columns?: GridTrack[]; grid_auto_flow?: GridAutoFlow; grid_row?: GridPlacement; grid_column?: GridPlacement }, s: Record<string, unknown>) {
  if (p.grid_template_rows !== undefined) s.gridTemplateRows = templateCss(who, "grid_template_rows", p.grid_template_rows);
  if (p.grid_template_columns !== undefined) s.gridTemplateColumns = templateCss(who, "grid_template_columns", p.grid_template_columns);
  if (p.grid_auto_rows !== undefined) s.gridAutoRows = p.grid_auto_rows.map((t) => trackCss(who, t)).join(" ");
  if (p.grid_auto_columns !== undefined) s.gridAutoColumns = p.grid_auto_columns.map((t) => trackCss(who, t)).join(" ");
  if (p.grid_auto_flow !== undefined) s.gridAutoFlow = flowCss(who, p.grid_auto_flow);
  if (p.grid_row !== undefined) s.gridRow = placementCss(who, "grid_row", p.grid_row);
  if (p.grid_column !== undefined) s.gridColumn = placementCss(who, "grid_column", p.grid_column);
}
