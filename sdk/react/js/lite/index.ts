// raea's live view runtime: React's API for the TSX a game is written in, without React's reconciler. Each frame the
// tree is rendered top-down (function components called, hooks kept per component instance), and the host elements
// it yields are matched to the ones already on the page by place and key: only the props, text and children that
// changed go out, as the same ops bevy-react's renderer sends (create, append, insert, remove, update, updateText),
// so Rust and the page see exactly what React would have made. No fibers, no scheduler, no work loop: one pass.
//
// A live game's bundle resolves "react" here (raea's bundle_composition, lite); renders keep React, and both
// must paint the same page (checked: raea play --snapshot against render --frame).
export { Fragment, jsx, jsxs, jsxDEV, createElement, isValidElement, cloneElement, Children, createContext, __at, __atk, __o, __v, __e, __h, __F, useContext, useState, useReducer, useMemo, useCallback, useRef, useId, useEffect, useLayoutEffect, useInsertionEffect, useSyncExternalStore, useImperativeHandle, useDebugValue, useTransition, useDeferredValue, memo, forwardRef, startTransition, Component, version, takeRendered, renderRoot } from "./parts/all";
export type { Element, Context } from "./parts/all";
export { default } from "./parts/root";


