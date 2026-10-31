// Pong's game (pure): the court, the rules, a step and the fold of keys into it, for pong's bench variants
// (edit.view.tsx, edit.stepfold.tsx) and seedling's. edit.tsx keeps its own copy: it's the play item the feed tweaks,
// and a tweak writes one file (raea-sys-tweak), so its Paddle and PADDLE_LEN are written out in it.
import { INPUT } from "./input";

// ---- sim:physics:start ------------------------------------------------- the game core (pure)
export const CW = 1920;
export const CH = 1080;
export const WALL_T = 66;
export const WALL_B = 1014;
export const END_L = 50;
export const END_R = 1870;
export const BOUNCE_L = 680;
export const BOUNCE_R = 1240;
export const PADDLE_LEN = 300;
export const PADDLE_TH = 56;
export const PXX = 190;
export const CXX = 1730;
export const BR = 28;
export const V0 = 50;
export const VMAX = 84;
export const P_SPEED = 50;
export const C_SPEED = 40;
export const TARGET = 3;

export type Contact = "paddleP" | "paddleC" | "wall" | "table" | "serve" | null;
export interface Mark {
  x: number;
  y: number;
  at: number;
  kind: "paddleP" | "paddleC" | "wall" | "table" | "serve" | "pointP" | "pointC";
}
export interface Game {
  bx: number;
  by: number;
  vx: number;
  vy: number;
  py: number;
  cy: number;
  sP: number;
  sC: number;
  serve: number;
  rally: number;
  phase: "play" | "pause" | "over";
  pause: number;
  contact: Contact;
  point: "P" | "C" | null;
  ended: number | null;
  human: boolean;
  humanAt: number | null;
  marks: Mark[];
}

export const INITIAL: Game = {
  bx: CW / 2, by: CH / 2, vx: 0, vy: 0,
  py: CH / 2, cy: CH / 2,
  sP: 0, sC: 0,
  serve: 0, rally: 0,
  phase: "pause", pause: 8,
  contact: null, point: null, ended: null,
  human: false, humanAt: null,
  marks: [],
};

export const clamp = (v: number, a: number, b: number) => (v < a ? a : v > b ? b : v);
export const seed = (n: number) => ((((n * 37 + 11) % 97) + 97) % 97) / 97;

/** one frame of the game. Pure: new object out, same input in every time. */
export function step(s: Game, keys: string[], frame: number): Game {
  const marks = s.marks.filter((m) => frame - m.at < 34);
  if (s.phase === "over") return { ...s, contact: null, marks };

  // the player's paddle: up and down
  const up = keys.indexOf("ArrowUp") >= 0 || keys.indexOf("w") >= 0 || keys.indexOf("W") >= 0;
  const down = keys.indexOf("ArrowDown") >= 0 || keys.indexOf("s") >= 0 || keys.indexOf("S") >= 0;
  const steer = (down ? 1 : 0) - (up ? 1 : 0);
  const py = clamp(s.py + steer * P_SPEED, WALL_T + PADDLE_LEN / 2, WALL_B - PADDLE_LEN / 2);

  // the opponent: chases the incoming ball with a seeded misjudgement that grows with the rally
  const chasing = s.phase === "play" && s.vx > 0;
  const aim = chasing ? s.by + (seed(s.serve * 5 + s.rally * 3) - 0.5) * (120 + Math.max(0, s.rally - 2) * 195) : CH / 2;
  const cy = clamp(s.cy + clamp(aim - s.cy, -C_SPEED, C_SPEED), WALL_T + PADDLE_LEN / 2, WALL_B - PADDLE_LEN / 2);

  // between points: the ball rests at the centre, then the serve snaps it away
  if (s.phase === "pause") {
    const pause = s.pause - 1;
    if (pause > 0) return { ...s, py, cy, pause, contact: null, point: s.point, marks };
    const dir = s.serve % 2 === 0 ? 1 : -1;
    const ang = (seed(s.serve * 11 + 3) - 0.5) * 2 * 12;
    return {
      ...s, py, cy,
      bx: CW / 2, by: CH / 2, vx: dir * Math.sqrt(V0 * V0 - ang * ang), vy: ang,
      phase: "play", pause: 0, rally: 0,
      contact: "serve", point: null,
      marks: [...marks, { x: CW / 2, y: CH / 2, at: frame, kind: "serve" }],
    };
  }

  // the ball
  let bx = s.bx + s.vx;
  let by = s.by + s.vy;
  let vx = s.vx;
  let vy = s.vy;
  let rally = s.rally;
  let contact: Contact = null;

  if (by - BR < WALL_T) {
    by = WALL_T + BR;
    vy = -vy;
    contact = "wall";
  } else if (by + BR > WALL_B) {
    by = WALL_B - BR;
    vy = -vy;
    contact = "wall";
  }
  const next: Mark[] = contact === "wall" ? [...marks, { x: bx, y: by, at: frame, kind: "wall" }] : marks;

  // the table bounce: once per trip, on the far side of the net
  const bouncedRight = vx > 0 && s.bx < BOUNCE_R && bx >= BOUNCE_R;
  const bouncedLeft = vx < 0 && s.bx > BOUNCE_L && bx <= BOUNCE_L;
  if (bouncedRight || bouncedLeft) {
    contact = "table";
    next.push({ x: bx, y: by, at: frame, kind: "table" });
  }

  // the player's paddle (the ball coming left)
  if (vx < 0 && bx - BR <= PXX + PADDLE_TH / 2 && s.bx - BR > PXX + PADDLE_TH / 2) {
    const off = (by - py) / (PADDLE_LEN / 2);
    if (Math.abs(off) <= 1 + BR / (PADDLE_LEN / 2)) {
      const speed = Math.min(Math.sqrt(vx * vx + vy * vy) * 1.05, VMAX);
      const nvy = clamp(off * speed * 0.62 + vy * 0.28, -speed * 0.85, speed * 0.85);
      vx = Math.sqrt(Math.max(speed * speed - nvy * nvy, speed * 0.45 * (speed * 0.45)));
      vy = nvy;
      bx = PXX + PADDLE_TH / 2 + BR;
      rally = s.rally + 1;
      contact = "paddleP";
      next.push({ x: bx, y: by, at: frame, kind: "paddleP" });
    }
  }

  // the opponent's paddle (the ball going right)
  if (vx > 0 && bx + BR >= CXX - PADDLE_TH / 2 && s.bx + BR < CXX - PADDLE_TH / 2) {
    const off = (by - cy) / (PADDLE_LEN / 2);
    if (Math.abs(off) <= 1 + BR / (PADDLE_LEN / 2)) {
      const speed = Math.min(Math.sqrt(vx * vx + vy * vy) * 1.05, VMAX);
      const nvy = clamp(off * speed * 0.62 + vy * 0.28, -speed * 0.85, speed * 0.85);
      vx = -Math.sqrt(Math.max(speed * speed - nvy * nvy, speed * 0.45 * (speed * 0.45)));
      vy = nvy;
      bx = CXX - PADDLE_TH / 2 - BR;
      rally = s.rally + 1;
      contact = "paddleC";
      next.push({ x: bx, y: by, at: frame, kind: "paddleC" });
    }
  }

  // a point: past either end
  const pastLeft = bx - BR < END_L;
  const pastRight = bx + BR > END_R;
  if (pastLeft || pastRight) {
    const who: "P" | "C" = pastLeft ? "C" : "P";
    const sP = who === "P" ? s.sP + 1 : s.sP;
    const sC = who === "C" ? s.sC + 1 : s.sC;
    const done = sP >= TARGET || sC >= TARGET;
    next.push({ x: bx, y: by, at: frame, kind: who === "P" ? "pointP" : "pointC" });
    return {
      ...s, py, cy,
      bx: CW / 2, by: CH / 2, vx: 0, vy: 0,
      sP, sC,
      serve: s.serve + 1, rally: 0,
      phase: done ? "over" : "pause",
      pause: done ? 0 : 18,
      contact: null,
      point: who,
      ended: done ? frame : null,
      marks: next,
    };
  }

  return {
    ...s, py, cy, bx, by, vx, vy, rally,
    phase: "play", contact, point: null, marks: next,
  };
}
// ---- sim:physics:end ---------------------------------------------------

export const EMPTY: string[] = [];
export const scripted = (keys: string[] | undefined, frame: number): string[] =>
  keys && keys.length ? keys : INPUT[frame] || EMPTY;

export const fold = (s: Game, i: { keys?: string[] }, f: number): Game => {
  const live = i && i.keys && i.keys.length ? i.keys : null;
  if (live && !s.human) {
    // the person takes over: their match begins from a fresh serve
    return { ...INITIAL, human: true, humanAt: f, phase: "pause", pause: 12 };
  }
  const keys = live ? live : s.human ? EMPTY : INPUT[f] || EMPTY;
  const out = step(s, keys, f);
  return s.human ? { ...out, human: true } : out;
};
