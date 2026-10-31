// S8b (docs/sdk.feature): a Blitz plugin added with one declaration. The plugin catalog
// (sdk/raea/bench/bevy-plugins/package.json) declares raea-bench-swatch (bench/bridge/swatch: a blitz-dom Widget, no
// Bevy in it) in raea.plugins with "widgets": { "x-swatch": ... }, so the plugin build of raea (raea-bevy-sdk motion)
// draws an <x-swatch> as Blitz would: its 200 x 150 box on the red page filled with its colour.
import { AbsoluteFill, Composition, registerRoot } from "raea";

declare module "react" {
  namespace JSX {
    interface IntrinsicElements {
      "x-swatch": { color?: string; style?: React.CSSProperties };
    }
  }
}

function Shot() {
  return (
    <AbsoluteFill style={{ background: "#ff0000" }}>
      <x-swatch color="#0000ff" style={{ position: "absolute", left: 100, top: 50, width: 200, height: 150 }} />
    </AbsoluteFill>
  );
}

registerRoot(() => <Composition id="bridgewidget" component={Shot} width={640} height={360} fps={30} durationInFrames={30} />);
