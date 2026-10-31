// raea/bevy: Bevy plugins driven from a TypeScript program. `BevyApp.open({ plugins })` makes a headless Bevy
// App inside raea's plugin build (raea-bevy-sdk), with the plugins named. The program steps it by frames (Time moves
// one frame per step, so a plugin steps exactly as it does under a render), and reaches it through bevy_remote's
// method table:
// - BRP's built-in methods, which components, resources, events, messages and queries go through;
// - raea's methods for states;
// - each plugin's own methods, such as avian3d/cast_ray or lightyear/input.
//
// Every value is typed by a plugin's generated handles: tools/bevy-plugins.sh gen writes <plugin>.gen.tsx from the
// built App's TypeRegistry, so `with(avian.RigidBody, "Bouncy")` fails to type-check. engine/bevy-plugins/src/bridge.rs
// is the other side; sdk/raea/docs/bevy-plugins.md explains the design.
export { component, resource, event, message, state, asset, with_, with, definePlugin, PluginGroup, PreStartup, Startup, PostStartup, First, PreUpdate, StateTransition, RunFixedMainLoop, FixedFirst, FixedPreUpdate, FixedUpdate, FixedPostUpdate, FixedLast, Update, SpawnScene, PostUpdate, Last, BevyApp, conditions, Transform, at } from "./bevy/all";
export type { Entity, Component, Resource, Event, Message, State, Handle, With, Plugin, PluginRef, OpenOptions, Condition, SystemOptions, OneShot, Then, Observing } from "./bevy/all";


