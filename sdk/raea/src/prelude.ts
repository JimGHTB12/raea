// docs/sdk.feature S1b (M1): a Bevy-named entry point, re-exporting what today lives in the files named. Sources move
// into it at M5; until then the old subpaths and this one export the same values.
// raea/prelude (bevy::prelude): every everyday name, 2D and 3D, in one import (docs/sdk.feature S1a).
export * from "./index";
export * from "./3d";
export * from "@raea/render/bevy3d";
// both export a Group (the 3D element; animate.tsx's plan type): the element wins, as a re-export (a const would keep
// 3D code in a 2D bundle, and live play opens a 3D engine for any bundle holding data-3d: runtime/live/src/lib.rs)
// /3d and Bevy's both name the lights: the prelude's are Bevy's, in Bevy's units (lux, lumens, cd/m²), as /light's
export { AmbientLight, DirectionalLight, PointLight, SpotLight, type AmbientLightProps, type DirectionalLightProps, type PointLightProps, type SpotLightProps } from "@raea/render/bevy3d";
