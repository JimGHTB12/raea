// The root, and React's default export.
import { flush, ROOT_ID } from "../../node_modules/bevy-react/src/bridge";
import { Children, Component, Fragment, Instance, Out, PRUNE_EVERY, ROOT_SLOT, S, children, cloneElement, createContext, createElement, effectsQueue, forwardRef, isValidElement, memo, prune, render, startTransition, sweep, unsee, useCallback, useContext, useDebugValue, useDeferredValue, useEffect, useId, useImperativeHandle, useInsertionEffect, useLayoutEffect, useMemo, useReducer, useRef, useState, useSyncExternalStore, useTransition, version } from "./all";

// ---- the root

const rootInstance: Instance = { fn: null, hooks: [], effects: [], cleanups: new Map(), kids: new Map(), seen: true, id: ROOT_SLOT, parent: null, el: null, inText: false, out: null, reads: null, codes: null, stale: false, misses: 0, cool: 0, sites: null, stamp: 0 };

const rootOut: Out[] = [];

const renderAgain = () => renderRoot(S.rootElement);

/** Render `element` into the page's root (the first call mounts; each later call patches what changed), then run the
 * effects the render set off and flush the frame's ops. */
export function renderRoot(element: any): void {
  S.rootElement = element;
  S.rerender = renderAgain;
  S.dirty = false;
  // (one map, one list and one re-render, kept: each was made again every frame)
  S.contexts.clear();
  S.readN = 0;
  S.codeN = 0;
  const out = rootOut;
  out.length = 0;
  S.pass++;
  S.bailout = (globalThis as any).__raea_lite_bailout !== false && S.pass % 128 !== 0;
  if (S.pass % PRUNE_EVERY === 0) prune(ROOT_SLOT, S.pass - PRUNE_EVERY);
  rootInstance.kids.forEach(unsee);
  render(element, rootInstance, ROOT_SLOT, out, false);
  rootInstance.kids.forEach(sweep, rootInstance.kids);
  S.rootHosts = children(ROOT_ID, S.rootHosts, out);
  flush();
  // (the queue kept at its size: splice(0) made a new array of every component rendered, each frame. An effect's
  // state change renders again in a microtask, after this loop)
  const n = S.effectsN;
  S.effectsN = 0;
  for (let i = 0; i < n; i++) {
    const inst = effectsQueue[i];
    effectsQueue[i] = undefined as any;
    // most components set off no effect this frame: nothing taken (splice made an empty array for each, every frame)
    if (inst.effects.length === 0) continue;
    const fx = inst.effects;
    inst.effects = [];
    for (const f of fx) {
      try {
        f();
      } catch (e) {
        console.error("[js] an effect threw:", e);
      }
    }
  }
}

const React = {
  Fragment,
  Children,
  Component,
  createElement,
  cloneElement,
  isValidElement,
  createContext,
  useContext,
  useState,
  useReducer,
  useMemo,
  useCallback,
  useRef,
  useId,
  useEffect,
  useLayoutEffect,
  useInsertionEffect,
  useSyncExternalStore,
  useImperativeHandle,
  useDebugValue,
  useTransition,
  useDeferredValue,
  memo,
  forwardRef,
  startTransition,
  version,
};

export default React;
