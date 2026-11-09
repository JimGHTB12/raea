// The plot.
import { SongEventKind, SongLevel, SongMap, check } from "./all";

// ---- the plot ----------------------------------------------------------------------------------------------------

const LEVEL_COLOUR: Record<SongLevel, string> = { intro: "#3a3f4b", low: "#2f5d8a", mid: "#b8862b", high: "#c2412d", outro: "#3a3f4b" };

const EVENT_COLOUR: Record<SongEventKind, string> = {
  build: "#f2c14e", drop: "#ff4d4d", stop: "#7fd1ff", hit: "#ffffff", dip: "#9b8cff", break: "#9b8cff",
};

export interface SongPlotProps {
  /** the song's map, from \`raea song\` */
  map: SongMap;
  /** in px */
  width?: number;
  /** in px */
  height?: number;
  /** the heading (the song's name, tempo, metre and length by default) */
  title?: string;
}

/** The map drawn for a human to check: section bands named, the energy curve with builds, stops, dips and drops on
 *  it, hits, the feature lanes, and the pacing (cuts per bar). `raea song FILE --plot OUT.png` renders it. */
export function SongPlot({ map, width = 2400, height = 1350, title }: SongPlotProps) {
  check(map);
  const L = 150;
  const R = 40;
  const W = width - L - R;
  const dur = Math.max(1e-6, map.duration);
  const x = (t: number) => L + (t / dur) * W;
  const beats = map.beats;
  const f = map.features;
  const font = "Helvetica, Arial, sans-serif";
  const rows = {
    title: [20, 70],
    sections: [90, 160],
    energy: [180, 640],
    loudness: [670, 760],
    low: [775, 865],
    onsets: [880, 970],
    brightness: [985, 1075],
    pace: [1100, 1230],
    axis: [1240, 1300],
  } as const;
  const sy = height / 1350;
  const Y = (v: number) => v * sy;
  const line = (v: readonly number[], top: number, bottom: number) =>
    beats.map((t, i) => `${i === 0 ? "M" : "L"}${x(t).toFixed(1)},${Y(bottom - (bottom - top) * (v[i] ?? 0)).toFixed(1)}`).join(" ");
  const area = (v: readonly number[], top: number, bottom: number) =>
    beats.length ? `${line(v, top, bottom)} L${x(beats[beats.length - 1]).toFixed(1)},${Y(bottom)} L${x(beats[0]).toFixed(1)},${Y(bottom)} Z` : "";
  const label = (text: string, left: number, top: number, size: number, colour = "#e8e8e8", weight = 400, extra: Record<string, unknown> = {}) => (
    <div key={`${text}-${left}-${top}`} style={{ position: "absolute", left, top: Y(top), fontFamily: font, fontSize: size * sy, color: colour, fontWeight: weight, whiteSpace: "nowrap", ...extra }}>{text}</div>
  );
  const [eT, eB] = rows.energy;
  const lanes: [keyof typeof rows, readonly number[], string, string][] = [
    ["loudness", f.loudness, "loudness", "#cfd8dc"],
    ["low", f.low, "low end", "#ff8a65"],
    ["onsets", f.onsets, "onsets", "#81c784"],
    ["brightness", f.brightness, "brightness", "#64b5f6"],
  ];
  const spans = map.events.filter((e) => e.end !== undefined);
  const instants = map.events.filter((e) => e.end === undefined);
  const secondsTicks: number[] = [];
  for (let t = 0; t <= dur; t += 10) secondsTicks.push(t);
  const [pT, pB] = rows.pace;
  const paceY = (c: number) => pB - ((Math.log2(c) + 2) / 4) * (pB - pT);
  const pacePath = map.bars
    .map((b, j) => {
      const end = map.bars[j + 1]?.time ?? dur;
      return `${j === 0 ? "M" : "L"}${x(b.time).toFixed(1)},${Y(paceY(b.cuts)).toFixed(1)} L${x(end).toFixed(1)},${Y(paceY(b.cuts)).toFixed(1)}`;
    })
    .join(" ");
  return (
    <div style={{ position: "absolute", left: 0, top: 0, width, height, background: "#111317", overflow: "hidden" }}>
      <svg width={width} height={height} viewBox={`0 0 ${width} ${height}`} style={{ position: "absolute", left: 0, top: 0 }}>
        {/* bars, faint */}
        {map.bars.map((b, j) => (
          <rect key={`bar${j}`} x={x(b.time)} y={Y(eT)} width={1} height={Y(rows.brightness[1] - eT)} fill={j % 4 === 0 ? "#2a2e36" : "#1b1e24"} />
        ))}
        {/* sections */}
        {map.sections.map((s, i) => (
          <rect key={`sec${i}`} x={x(s.time)} y={Y(rows.sections[0])} width={Math.max(1, x(s.end) - x(s.time) - 2)} height={Y(rows.sections[1] - rows.sections[0])} fill={LEVEL_COLOUR[s.level] ?? "#444"} />
        ))}
        {map.sections.map((s, i) => (
          <rect key={`secline${i}`} x={x(s.time)} y={Y(rows.sections[1])} width={2} height={Y(rows.pace[1] - rows.sections[1])} fill="#ffffff" opacity={0.18} />
        ))}
        {/* spans on the energy lane */}
        {spans.map((e, i) => {
          const x0 = x(e.time);
          const x1 = x(e.end ?? e.time);
          if (e.kind === "build") {
            return <path key={`span${i}`} d={`M${x0},${Y(eB)} L${x1},${Y(eT)} L${x1},${Y(eB)} Z`} fill={EVENT_COLOUR.build} opacity={0.28} />;
          }
          return <rect key={`span${i}`} x={x0} y={Y(eT)} width={Math.max(2, x1 - x0)} height={Y(eB - eT)} fill={EVENT_COLOUR[e.kind]} opacity={e.kind === "stop" ? 0.3 : 0.18} />;
        })}
        {/* energy */}
        <path d={area(f.energy, eT, eB)} fill="#ffffff" opacity={0.08} />
        <path d={line(f.energy, eT, eB)} fill="none" stroke="#ffffff" strokeWidth={3 * sy} />
        {/* the feature lanes */}
        {lanes.map(([row, v, , colour]) => {
          const [t, b] = rows[row];
          return <path key={`lane${row}`} d={area(v, t, b)} fill={colour} opacity={0.75} />;
        })}
        {/* drops and hits */}
        {instants.map((e, i) =>
          e.kind === "drop" ? (
            <g key={`ev${i}`}>
              <rect x={x(e.time) - 2} y={Y(eT - 10)} width={4} height={Y(rows.brightness[1] - eT + 10)} fill={EVENT_COLOUR.drop} opacity={0.4 + 0.6 * e.strength} />
              <path d={`M${x(e.time) - 12},${Y(eT - 26)} L${x(e.time) + 12},${Y(eT - 26)} L${x(e.time)},${Y(eT - 4)} Z`} fill={EVENT_COLOUR.drop} />
            </g>
          ) : (
            <rect key={`ev${i}`} x={x(e.time) - 1.5} y={Y(eB - 40 - 50 * e.strength)} width={3} height={Y(40 + 50 * e.strength)} fill={EVENT_COLOUR.hit} opacity={0.9} />
          ),
        )}
        {spans.filter((e) => e.kind === "stop").flatMap((e, i) =>
          (e.hits ?? []).map((h, k) => <circle key={`sh${i}-${k}`} cx={x(h.time)} cy={Y(eB - 20)} r={4 * sy} fill={EVENT_COLOUR.stop} />),
        )}
        {/* pacing */}
        {[0.25, 0.5, 1, 2, 4].map((c) => (
          <rect key={`pg${c}`} x={L} y={Y(paceY(c))} width={W} height={1} fill="#2a2e36" />
        ))}
        <path d={pacePath} fill="none" stroke="#f2c14e" strokeWidth={3 * sy} />
        {/* axis */}
        {secondsTicks.map((t) => (
          <rect key={`tick${t}`} x={x(t)} y={Y(rows.axis[0])} width={1} height={Y(t % 30 === 0 ? 16 : 8)} fill="#8a8f98" />
        ))}
      </svg>
      {label(title ?? `${map.source ?? "song"}  ·  ${map.tempo ?? "?"} BPM  ·  ${map.beatsPerBar}/4  ·  ${Math.floor(dur / 60)}:${String(Math.round(dur % 60)).padStart(2, "0")}  ·  sections from ${map.sectionsFrom ?? "?"}`, L, rows.title[0], 34, "#ffffff", 700)}
      {label("build", width - R - 640, rows.title[0] + 8, 22, EVENT_COLOUR.build, 700)}
      {label("drop", width - R - 540, rows.title[0] + 8, 22, EVENT_COLOUR.drop, 700)}
      {label("stop · its hits", width - R - 450, rows.title[0] + 8, 22, EVENT_COLOUR.stop, 700)}
      {label("hit", width - R - 270, rows.title[0] + 8, 22, EVENT_COLOUR.hit, 700)}
      {label("dip / break", width - R - 200, rows.title[0] + 8, 22, EVENT_COLOUR.dip, 700)}
      {map.sections.map((s) => label(s.name, x(s.time) + 8, rows.sections[0] + 8, 22, "#ffffff", 700, { maxWidth: Math.max(0, x(s.end) - x(s.time) - 14), overflow: "hidden" }))}
      {map.sections.map((s) => label(`${s.time.toFixed(1)}s · ${s.level} · ${s.cuts}/bar`, x(s.time) + 8, rows.sections[0] + 40, 16, "#e0e0e0", 400, { maxWidth: Math.max(0, x(s.end) - x(s.time) - 14), overflow: "hidden" }))}
      {instants.filter((e) => e.kind === "drop").map((e) => label(`${e.time.toFixed(1)}`, x(e.time) + 6, eT + 2, 18, EVENT_COLOUR.drop, 700))}
      {spans.filter((e) => e.kind === "build").map((e, i) => label(`build ${e.time.toFixed(1)}-${(e.end ?? 0).toFixed(1)}`, x(e.time) + 4, eB - 26 - (i % 2) * 22, 15, EVENT_COLOUR.build, 700))}
      {spans.filter((e) => e.kind === "dip" || e.kind === "break").map((e) => label(`${e.kind} ${e.time.toFixed(1)}`, x(e.time) + 4, eT + 6, 15, EVENT_COLOUR.dip, 700))}
      {label("energy", 16, eT, 22, "#ffffff", 700)}
      {lanes.map(([row, , name, colour]) => label(name, 16, rows[row][0] + 20, 20, colour, 700))}
      {label("cuts / bar", 16, pT + 10, 20, "#f2c14e", 700)}
      {[0.25, 1, 4].map((c) => label(String(c), L - 46, paceY(c) - 10, 15, "#8a8f98"))}
      {secondsTicks.filter((t) => t % 30 === 0).map((t) => label(`${Math.floor(t / 60)}:${String(t % 60).padStart(2, "0")}`, x(t) - 18, rows.axis[0] + 20, 18, "#8a8f98"))}
    </div>
  );
}
