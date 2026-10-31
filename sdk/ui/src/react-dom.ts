// react-dom (and react-dom/client) as @raea/ui provides them on Bevy (docs/ts-native.feature P10): createRoot,
// createPortal and flushSync over the React adapter's DOM mode (react.ts), the form hooks React DOM exports, and the
// resource hints (no-ops: nothing loads stylesheets or scripts here). What React DOM has that isn't here: hydrateRoot
// (no server rendering), findDOMNode and render (removed in React 19), and requestFormReset, which React's reconciler
// doesn't export for another host (a form still resets itself after its action).
import * as React from "react";
import { createPortal, createRoot, flushSync } from "./react";

export { createPortal, createRoot, flushSync };
export const version = "19.2.0";
type Dispatcher = { useHostTransitionStatus(): unknown; useFormState?: unknown };
const H = () => (React as unknown as { __CLIENT_INTERNALS_DO_NOT_USE_OR_WARN_USERS_THEY_CANNOT_UPGRADE: { H: Dispatcher | null } }).__CLIENT_INTERNALS_DO_NOT_USE_OR_WARN_USERS_THEY_CANNOT_UPGRADE.H;
/** The status of the form this component is in: pending, its data, method and action while its action runs. */
export function useFormStatus(): { pending: boolean; data: FormData | null; method: string | null; action: unknown } {
  const d = H();
  if (!d) throw new Error("Invalid hook call. Hooks can only be called inside of the body of a function component.");
  return d.useHostTransitionStatus() as never;
}
/** The pre-19 name of useActionState. */
export const useFormState = React.useActionState;
export const unstable_batchedUpdates = <A, R>(fn: (a: A) => R, a: A): R => fn(a);
export function requestFormReset(_form: unknown): void {
  throw new Error("requestFormReset isn't available on @raea/ui's host (a form resets itself after its action)");
}
export const preconnect = (_href: string, _o?: unknown) => {};
export const prefetchDNS = (_href: string) => {};
export const preinit = (_href: string, _o?: unknown) => {};
export const preinitModule = (_href: string, _o?: unknown) => {};
export const preload = (_href: string, _o?: unknown) => {};
export const preloadModule = (_href: string, _o?: unknown) => {};
export default { createPortal, createRoot, flushSync, version, useFormStatus, useFormState, unstable_batchedUpdates, requestFormReset, preconnect, prefetchDNS, preinit, preinitModule, preload, preloadModule };
