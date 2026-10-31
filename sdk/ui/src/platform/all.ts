// Every part of platform.ts, in the order they load: each after what it uses while it loads. The parts import
// each other only from here, so this order holds whichever is reached first.
export * from "./core";
export * from "./events";
export * from "./text";
export * from "./url";
export * from "./fetch";
export * from "./audio";
export * from "./storage";
export * from "./blob";
