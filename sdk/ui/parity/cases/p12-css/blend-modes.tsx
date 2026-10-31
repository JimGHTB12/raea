// mix-blend-mode, every mode, a coloured disc over a two-tone striped ground; isolation: isolate keeps a group's
// blending inside it; background-blend-mode mixes a box's gradient with its colour.
// proves: css.properties.mix-blend-mode, css.properties.mix-blend-mode.normal, css.properties.mix-blend-mode.multiply, css.properties.mix-blend-mode.screen, css.properties.mix-blend-mode.overlay, css.properties.mix-blend-mode.darken, css.properties.mix-blend-mode.lighten, css.properties.mix-blend-mode.color-dodge, css.properties.mix-blend-mode.color-burn, css.properties.mix-blend-mode.hard-light, css.properties.mix-blend-mode.soft-light, css.properties.mix-blend-mode.difference, css.properties.mix-blend-mode.exclusion, css.properties.mix-blend-mode.hue, css.properties.mix-blend-mode.saturation, css.properties.mix-blend-mode.color, css.properties.mix-blend-mode.luminosity, css.properties.mix-blend-mode.plus-lighter, css.properties.isolation, css.properties.isolation.isolate, css.properties.isolation.auto, css.properties.background-blend-mode, css.properties.background-blend-mode.multiply, css.properties.background-blend-mode.screen, css.properties.background-blend-mode.overlay, css.properties.background-blend-mode.difference, css.types.blend-mode
export const steps = [];
const MODES = ["normal", "multiply", "screen", "overlay", "darken", "lighten", "color-dodge", "color-burn", "hard-light", "soft-light", "difference", "exclusion", "hue", "saturation", "color", "luminosity", "plus-lighter"] as const;
const GROUND = "repeating-linear-gradient(90deg, #1e3a8a 0 22px, #fbbf24 22px 44px)";
export default function Case() {
  return (
    <div data-testid="root" className="relative h-full w-full bg-white">
      {MODES.map((m, i) => (
        <div key={m} data-testid={`m-${m}`} className="absolute" style={{ left: 16 + (i % 6) * 128, top: 16 + Math.floor(i / 6) * 128, width: 112, height: 112, background: GROUND }}>
          <div className="absolute rounded-full" style={{ left: 16, top: 16, width: 80, height: 80, background: "rgb(220, 60, 140)", mixBlendMode: m }} />
        </div>
      ))}
      <div data-testid="isolated" className="absolute" style={{ left: 16, top: 410, width: 240, height: 160, background: GROUND }}>
        <div className="absolute" style={{ left: 20, top: 20, width: 200, height: 120, isolation: "isolate" }}>
          <div className="absolute rounded-full" style={{ left: 0, top: 0, width: 120, height: 120, background: "#22d3ee", mixBlendMode: "multiply" }} />
          <div className="absolute rounded-full" style={{ left: 80, top: 0, width: 120, height: 120, background: "#f43f5e", mixBlendMode: "multiply" }} />
        </div>
      </div>
      <div data-testid="not-isolated" className="absolute" style={{ left: 272, top: 410, width: 240, height: 160, background: GROUND }}>
        <div className="absolute" style={{ left: 20, top: 20, width: 200, height: 120, isolation: "auto" }}>
          <div className="absolute rounded-full" style={{ left: 0, top: 0, width: 120, height: 120, background: "#22d3ee", mixBlendMode: "multiply" }} />
          <div className="absolute rounded-full" style={{ left: 80, top: 0, width: 120, height: 120, background: "#f43f5e", mixBlendMode: "multiply" }} />
        </div>
      </div>
      {(["multiply", "screen", "overlay", "difference"] as const).map((m, i) => (
        <div key={m} data-testid={`bg-${m}`} className="absolute" style={{ left: 528 + (i % 2) * 130, top: 410 + Math.floor(i / 2) * 84, width: 120, height: 76, backgroundColor: "#7c3aed", backgroundImage: "linear-gradient(90deg, #fde047, #0ea5e9)", backgroundBlendMode: m }} />
      ))}
    </div>
  );
}
