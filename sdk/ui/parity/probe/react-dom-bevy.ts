// "react-dom" and "react-dom/client" as the Bevy bundle resolves them: @raea/ui's react-dom (sdk/ui/src/react-dom.ts),
// over its React adapter's DOM mode (docs/ts-native.feature P18). What it doesn't provide is absent, so a case using it
// fails on Bevy's side.
export * from "@raea/ui/react-dom";
export { default } from "@raea/ui/react-dom";
