// `raea init --layout feature`: an object's folder laid out as bevy_new_2d's template is, by feature (docs/sdk.feature
// S5a). A video is an App like a game: main.tsx registers its Composition, each feature (here its title) is one file
// with its components, shots are scenes in Sequences, and files sit in assets/, where staticFile reads them.

export interface Shape { width: number; height: number; fps: number }

/** The folders it makes, empty ones included: assets/ and src/'s screens/, shots/ and ui/. */
export const FEATURE_DIRS = ["assets", "src", "src/screens", "src/shots", "src/ui"];

/** Its files by path, for a project called `name`. */
export function featureLayout(name: string, shape: Shape): Record<string, string> {
  const { width, height, fps } = shape;
  return {
    "space.json": JSON.stringify({ name, kind: "video" }, null, 2) + "\n",
    "package.json": JSON.stringify({ name, private: true, raea: { plugins: [] } }, null, 2) + "\n",
    "src/main.tsx": `// The App: its plugins and its root. A video registers its Composition here; its shots play in Sequences.
import { AbsoluteFill, Composition, Sequence, registerRoot } from "raea";
import { Opening } from "./shots/opening";
import { Title } from "./title";

const FPS = ${fps};

function Main() {
  return (
    <AbsoluteFill style={{ background: "#101418" }}>
      <Sequence from={0} durationInFrames={FPS * 3}>
        <Opening />
      </Sequence>
      <Title text="${name}" from={FPS / 2} />
    </AbsoluteFill>
  );
}

registerRoot(() => <Composition id="main" component={Main} width={${width}} height={${height}} fps={FPS} durationInFrames={FPS * 3} />);
`,
    "src/title.tsx": `// A feature: its components together in one file. A feature with work in the App each frame (systems, resources)
// also exports its plugin, definePlugin({ name, build(app) }) from "raea/app"; this one draws only, so it has none.
import { interpolate, useCurrentFrame } from "raea";

export function Title({ text, from }: { text: string; from: number }) {
  const f = useCurrentFrame();
  const shown = interpolate(f, [from, from + 12], [0, 1], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });
  return (
    <div style={{ position: "absolute", left: 0, right: 0, bottom: "12%", textAlign: "center", fontFamily: "Inter", fontWeight: 800, fontSize: ${Math.round(Math.min(width, height) / 12)}, color: "#ffffff", opacity: shown }}>
      {text}
    </div>
  );
}
`,
    "src/shots/opening.tsx": `// A shot: a scene component, one time range of the cut (main.tsx places it in a Sequence).
import { AbsoluteFill, interpolate, useCurrentFrame } from "raea";

export function Opening() {
  const f = useCurrentFrame();
  const x = interpolate(f, [0, 60], [-20, 20], { extrapolateRight: "clamp" });
  return (
    <AbsoluteFill style={{ alignItems: "center", justifyContent: "center" }}>
      <div style={{ width: "30%", aspectRatio: "1", borderRadius: "50%", background: "#ff7a1a", transform: \`translateX(\${x}%)\` }} />
    </AbsoluteFill>
  );
}
`,
    "AGENTS.md": `# This project

A video, laid out by feature as a Bevy project is (raea init --layout feature):

- space.json: its name and kind. package.json: raea.plugins, the Bevy, Blitz and raea plugins it uses.
- assets/: its files. \`staticFile("x.png")\` reads assets/x.png (a media/ folder beside it is read first).
- src/main.tsx: the App, rendered with \`raea render src/main.tsx\`. It registers the Composition.
- src/<feature>.tsx: one feature each (title, captions, music), its components together; one with work in the App
  each frame also exports its plugin (definePlugin from "raea/app").
- src/shots/: the shots, each a scene component placed in a Sequence. src/screens/: one file per top-level state.
  src/ui/: page parts shared between features.

Never a components/ folder: a feature file holds a feature's components by name.
`,
  };
}
