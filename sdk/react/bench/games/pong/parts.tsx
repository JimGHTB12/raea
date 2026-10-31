// Pong's score and paddles.
import { PADDLE_LEN, PADDLE_TH } from "./game";

export function Score({ n, x, y, live, align }: { n: number; x: number; y: number; live: boolean; align: "left" | "right" }) {
  return (
    <div
      style={{
        position: "absolute", left: x, top: y, width: 320, textAlign: align,
        fontFamily: "var(--brand-font-heading)", fontWeight: 700, lineHeight: 1,
        fontSize: "var(--brand-h1-size)", letterSpacing: "0.01em",
        color: "var(--brand-text)", opacity: live ? 1 : 0.85,
        fontVariantNumeric: "tabular-nums",
        transform: `scale(${live ? 1.1 : 1})`, transformOrigin: align === "left" ? "0 50%" : "100% 50%",
      }}
    >
      {n}
    </div>
  );
}

export function Paddle({
  x, y, face, edge, flash, lean, edgeLeft,
}: {
  x: number; y: number; face: string; edge: string; flash: number; lean: number; edgeLeft: boolean;
}) {
  return (
    <div
      style={{
        position: "absolute", left: x - PADDLE_TH / 2, top: y - PADDLE_LEN / 2,
        width: PADDLE_TH, height: PADDLE_LEN,
        background: face, borderRadius: "var(--brand-radius)",
        transform: `rotate(${lean}deg) scale(${1 + flash * 0.05})`,
        boxShadow: "0 14px 0 rgba(3,20,15,0.24)",
      }}
    >
      <div
        style={{
          position: "absolute", top: 18, bottom: 18, width: 9, borderRadius: 5,
          left: edgeLeft ? 8 : undefined, right: edgeLeft ? undefined : 8,
          background: edge, opacity: 0.5 + flash * 0.5,
        }}
      />
    </div>
  );
}
