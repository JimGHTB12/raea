// A frame held while its R3F scene waits on a load (drei's <Environment>, useGLTF, useTexture: whatever suspends). The
// scene renders under a Suspense whose fallback opens a hold on raea's frame (the entry's HOLD, runtime/bundle
// entry.rs): the frame isn't marked done while one is open, so a render never takes a frame from before its loads came
// (the first frame of a film was the 2D alone). The last release wakes the frame: <R3F> draws it again with what loaded.
import { Suspense, useEffect, useLayoutEffect, useState, type ReactNode } from "react";

interface Hold {
  n: number;
  wake: Set<() => void>;
}

const HOLD = ((globalThis as { __raeaHold?: Hold }).__raeaHold ??= { n: 0, wake: new Set() });

function Waiting() {
  useLayoutEffect(() => {
    HOLD.n++;
    return () => {
      HOLD.n = Math.max(0, HOLD.n - 1);
      if (HOLD.n === 0) for (const w of [...HOLD.wake]) w();
    };
  }, []);
  return null;
}

/** `children` under a Suspense that holds raea's frame while any of it suspends */
export function Held({ children }: { children?: ReactNode }) {
  return <Suspense fallback={<Waiting />}>{children}</Suspense>;
}

/** a count that goes up each time raea's frame is woken: the component reading it draws the frame again */
export function useWake(): number {
  const [n, setN] = useState(0);
  useEffect(() => {
    const w = () => setN((x) => x + 1);
    HOLD.wake.add(w);
    return () => void HOLD.wake.delete(w);
  }, []);
  return n;
}
