// docs/sdk.feature S1b (M1): a Bevy-named entry point, re-exporting what today lives in the files named. Sources move
// into it at M5; until then the old subpaths and this one export the same values.
// raea/app (bevy_app): the App, its plugins and schedules; a video registers its Composition here.
export * from "@raea/video/registry";
// a headless App reached through bevy_remote (bevy.ts): App is Bevy's name for it
export { BevyApp as App, BevyApp, definePlugin, PluginGroup, type Plugin, type PluginRef, type OpenOptions, type SystemOptions, type Condition, conditions } from "@raea/app/bevy";
export { PreStartup, Startup, PostStartup, First, PreUpdate, StateTransition, RunFixedMainLoop, FixedFirst, FixedPreUpdate, FixedUpdate, FixedPostUpdate, FixedLast, Update, SpawnScene, PostUpdate, Last } from "@raea/app/bevy";
