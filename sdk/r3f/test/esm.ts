// bun reads @react-three/fiber's `main` (its CommonJS build), which requires three's CommonJS build: a second three
// beside the ES one, whose classes R3F's checks (instanceof THREE.Color) then don't know. raea's bundler takes R3F's
// `module` (ES) build, as Vite does; here the CommonJS file is loaded as the ES build's source, so the tests do too.
import { plugin } from "bun";
import { readFileSync } from "node:fs";

plugin({
  name: "r3f-es",
  setup(b) {
    b.onLoad({ filter: /@react-three\/fiber\/dist\/react-three-fiber\.cjs\.js$/ }, (a) => ({
      contents: readFileSync(a.path.replace(/\.cjs\.js$/, ".esm.js"), "utf8"),
      loader: "js",
    }));
  },
});
