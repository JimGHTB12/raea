// PONG on bevy_seedling: sdk/react/bench/games/pong with every sound played by seedling on the one mixer core
// (raea/seedling, raea-seedling), the hall's room tone looping through raea's Freeverb as a seedling node. Its
// render's bounce and a play's tap must be the same samples, and its frames the same pixels (sdk/react/bench/games/audio.sh,
// replay.sh). The pong it copies:
// PONG — a table-tennis table seen from directly overhead in an empty night hall,
// turned sideways into a classic court: you play the left paddle, the computer the right.
// The game is a pure fold: live arrow keys when someone plays, the scripted attract
// match (input.ts) when nobody does. The court is drawn in its own units and scaled
// to fill whatever stage it is given.
import {
  Composition, Easing, Mix, Sequence, interpolate, registerRoot, useCurrentFrame, useGameEvents, useGameState, useInput, useVideoConfig,
} from "raea";
import { AbsoluteFill } from "raea";
import { Seedling } from "raea/seedling";

const db = (g: number) => Math.pow(10, g / 20);
import { css } from "./styles.gen";
import { INPUT } from "./pong/input";
import { CW, CH, PXX, CXX, BR, TARGET, INITIAL, clamp, EMPTY, fold, type Mark } from "./pong/game";
import { STEPS } from "./pong/steps";
import { POCK, TOCK, THOCK, TAK, CLACK, SCUFF, BELL, HALL } from "./pong/sounds";
import { Score, Paddle } from "./pong/parts";

function Sounds({
  hits, walls, tables, points, over, steps,
}: {
  hits: number[]; walls: number[]; tables: number[]; points: number[]; over: number[]; steps: number[];
}) {
  return (
    <>
      <Sequence from={0} layout="none">
        <Seedling src={HALL} id={"hall"} volume={db(-22)} loop fx={[{ kind: "Freeverb", options: { roomSize: 0.6, dampening: 4000, wet: 0.25 } }]} />
      </Sequence>
      {hits.map((f, i) => (
        <Sequence key={"h" + i} from={f} layout="none">
          <Seedling src={i % 2 ? TOCK : POCK} id={"h" + i} volume={db(i % 2 ? -6 : -4)} />
        </Sequence>
      ))}
      {walls.map((f, i) => (
        <Sequence key={"w" + i} from={f} layout="none">
          <Seedling src={THOCK} id={"w" + i} volume={db(-9)} />
        </Sequence>
      ))}
      {tables.map((f, i) => (
        <Sequence key={"t" + i} from={f} layout="none">
          <Seedling src={TAK} id={"t" + i} volume={db(-9)} />
        </Sequence>
      ))}
      {steps.map((f, i) => (
        <Sequence key={"s" + i} from={f} layout="none">
          <Seedling src={SCUFF} id={"s" + i} volume={db(-14)} />
        </Sequence>
      ))}
      {points.map((f, i) => (
        <Sequence key={"p" + i} from={f} layout="none">
          <Seedling src={CLACK} id={"p" + i} volume={db(-5)} />
        </Sequence>
      ))}
      {over.map((f, i) => (
        <Sequence key={"o" + i} from={f} layout="none">
          <Seedling src={BELL} id={"o" + i} volume={db(-7)} />
        </Sequence>
      ))}
    </>
  );
}

function Pong() {
  const frame = useCurrentFrame();
  const stage = useVideoConfig();
  const inp = useInput();
  const g = useGameState(INITIAL, fold);

  const hits = useGameEvents(INITIAL, fold, (b, a) => a.contact !== null && b.contact === null && a.contact !== "wall" && a.contact !== "table");
  const walls = useGameEvents(INITIAL, fold, (b, a) => a.contact === "wall" && b.contact !== "wall");
  const tables = useGameEvents(INITIAL, fold, (b, a) => a.contact === "table" && b.contact !== "table");
  const points = useGameEvents(INITIAL, fold, (b, a) => a.point !== null && b.point === null);
  const over = useGameEvents(INITIAL, fold, (b, a) => a.phase === "over" && b.phase !== "over");

  const scale = Math.min(stage.width / CW, stage.height / CH);
  const ox = (stage.width - CW * scale) / 2;
  const oy = (stage.height - CH * scale) / 2;

  const flashOf = (kind: Mark["kind"]) => {
    let best = 0;
    for (const m of g.marks) {
      const age = frame - m.at;
      if (m.kind === kind && age >= 0 && age < 6) best = Math.max(best, 1 - age / 6);
    }
    return best;
  };
  const steerLean = (g.bx > 0 && (INPUT[frame] || EMPTY)[0] === "ArrowDown") || (inp && inp.keys && inp.keys.indexOf("ArrowDown") >= 0) ? 1.6 : 0;

  // the outcome, and when it landed
  const won = g.sP >= TARGET || g.sC >= TARGET;
  const endAt = g.ended !== null && g.ended <= frame ? g.ended : frame - 6;
  const cardLocal = won ? Math.max(0, frame - endAt) : 0;
  const cardO = won ? Math.min(1, cardLocal / 6) : 0;
  const cardRise = won ? (1 - Math.min(1, cardLocal / 8)) * 46 : 0;

  const hintShow = frame < 78 || (g.human && g.humanAt !== null && frame - g.humanAt < 90);
  const hintO = hintShow ? (frame < 60 ? 1 : Math.max(0, 1 - (frame - 60) / 18)) : 0;

  return (
    <AbsoluteFill style={{ backgroundColor: "var(--brand-background)" }}>
      <style>{css}</style>

      {/* the court, scaled to fill the stage */}
      <div
        style={{
          position: "absolute", left: ox, top: oy, width: CW, height: CH,
          transform: `scale(${scale})`, transformOrigin: "top left",
        }}
      >
        {/* the hall's light: a pool on the table, the corners falling away */}
        <div
          style={{
            position: "absolute", left: 0, top: 0, width: CW, height: CH,
            background:
              "radial-gradient(118% 60% at 50% 46%, rgba(244,234,214,0.13), rgba(244,234,214,0.02) 58%, rgba(3,20,15,0.24) 100%)",
          }}
        />

        {/* the table's paint */}
        <div
          style={{
            position: "absolute", left: 40, top: 28, width: CW - 80, height: CH - 56,
            border: "10px solid rgba(244,234,214,0.6)", borderRadius: 10,
          }}
        />
        <div
          style={{
            position: "absolute", left: 50, top: CH / 2 - 4, width: CW - 100, height: 8,
            background: "repeating-linear-gradient(90deg, rgba(244,234,214,0.34) 0 38px, rgba(244,234,214,0) 38px 68px)",
          }}
        />
        {/* the net */}
        <div style={{ position: "absolute", left: CW / 2 - 16, top: 28, width: 32, height: CH - 56, background: "rgba(244,234,214,0.18)" }} />
        <div style={{ position: "absolute", left: CW / 2 - 5, top: 28, width: 10, height: CH - 56, background: "rgba(244,234,214,0.75)" }} />
        <div style={{ position: "absolute", left: CW / 2 - 24, top: 14, width: 48, height: 26, background: "var(--brand-ink)", borderRadius: 5 }} />
        <div style={{ position: "absolute", left: CW / 2 - 24, top: CH - 40, width: 48, height: 26, background: "var(--brand-ink)", borderRadius: 5 }} />

        {/* the hint */}
        <div
          style={{
            position: "absolute", left: 0, top: 150, width: CW, textAlign: "center",
            fontFamily: "var(--brand-font-heading)", fontWeight: 700, fontSize: "var(--brand-h3-size)",
            letterSpacing: "0.16em", color: "var(--brand-text)", opacity: hintO,
          }}
        >
          {g.human ? "YOUR MATCH — ARROW KEYS TO MOVE" : "FIRST TO 3"}
        </div>

        {/* the score, painted on the table */}
        <Score n={g.sP} x={170} y={260} live={g.point === "P"} align="left" />
        <Score n={g.sC} x={CW - 490} y={260} live={g.point === "C"} align="right" />

        {/* contact rings */}
        {g.marks.map((m, i) => {
          const age = frame - m.at;
          const isPoint = m.kind === "pointP" || m.kind === "pointC";
          const isTap = m.kind === "table";
          const life = isPoint ? 32 : isTap ? 18 : 22;
          const r = interpolate(age, [0, life], [BR, isPoint ? 210 : isTap ? 84 : 104], {
            extrapolateLeft: "clamp", extrapolateRight: "clamp", easing: Easing.out(Easing.cubic),
          });
          const o = interpolate(age, [0, life], [isPoint ? 0.7 : 0.85, 0], {
            extrapolateLeft: "clamp", extrapolateRight: "clamp",
          });
          return (
            <div
              key={i}
              style={{
                position: "absolute", left: m.x - r, top: m.y - r, width: r * 2, height: r * 2,
                borderRadius: "50%", opacity: o,
                border: `7px solid ${isPoint ? "var(--brand-accent)" : "var(--brand-text)"}`,
              }}
            />
          );
        })}

        {/* the ball: ghosts, shadow, then the ball itself */}
        {g.phase === "play" ? (
          <>
            <div
              style={{
                position: "absolute", left: g.bx - g.vx * 1.15 * 3 - BR, top: g.by - g.vy * 1.15 * 3 - BR,
                width: BR * 2, height: BR * 2, borderRadius: "50%", background: "var(--brand-text)", opacity: 0.06,
              }}
            />
            <div
              style={{
                position: "absolute", left: g.bx - g.vx * 1.15 * 2 - BR, top: g.by - g.vy * 1.15 * 2 - BR,
                width: BR * 2, height: BR * 2, borderRadius: "50%", background: "var(--brand-text)", opacity: 0.1,
              }}
            />
            <div
              style={{
                position: "absolute", left: g.bx - g.vx * 1.15 - BR, top: g.by - g.vy * 1.15 - BR,
                width: BR * 2, height: BR * 2, borderRadius: "50%", background: "var(--brand-text)", opacity: 0.16,
              }}
            />
          </>
        ) : null}
        <div
          style={{
            position: "absolute", left: g.bx + 13 - BR, top: g.by + 18 - BR, width: BR * 2, height: BR * 2,
            borderRadius: "50%", background: "rgba(3,20,15,0.3)",
          }}
        />
        <div
          style={{
            position: "absolute", left: g.bx - BR, top: g.by - BR, width: BR * 2, height: BR * 2,
            borderRadius: "50%", background: "var(--brand-text)",
          }}
        />

        {/* the opponent's paddle, then the player's */}
        <Paddle x={CXX} y={g.cy} face="var(--brand-ink)" edge="var(--brand-color-3)" flash={flashOf("paddleC")} lean={0} edgeLeft={false} />
        <Paddle x={PXX} y={g.py} face="var(--brand-accent)" edge="var(--brand-text)" flash={flashOf("paddleP")} lean={steerLean} edgeLeft />

        {/* the outcome */}
        {won ? (
          <div style={{ position: "absolute", left: 0, top: 0, width: CW, height: CH, background: "rgba(3,20,15,0.42)", opacity: cardO }}>
            <div
              style={{
                position: "absolute", left: CW / 2 - 430, top: CH / 2 - 260 + cardRise, width: 860, height: 520,
                background: "var(--brand-accent)", borderRadius: "var(--brand-radius)",
                display: "flex", flexDirection: "column", alignItems: "center", justifyContent: "center",
                boxShadow: "0 30px 70px rgba(3,20,15,0.5)",
              }}
            >
              <div
                style={{
                  fontFamily: "var(--brand-font-heading)", fontWeight: 700, lineHeight: 1,
                  fontSize: "var(--brand-h3-size)", letterSpacing: "0.16em", color: "var(--brand-on-accent)",
                }}
              >
                {g.sP > g.sC ? "YOU WIN" : "YOU LOSE"}
              </div>
              <div
                style={{
                  marginTop: 34, fontFamily: "var(--brand-font-heading)", fontWeight: 700, lineHeight: 1,
                  fontSize: "var(--brand-h1-size)", letterSpacing: "0.01em", color: "var(--brand-on-accent)",
                  fontVariantNumeric: "tabular-nums",
                }}
              >
                {g.sP} — {g.sC}
              </div>
            </div>
          </div>
        ) : null}
      </div>

      <Sounds hits={hits} walls={walls} tables={tables} points={points} over={over} steps={STEPS} />
      <Mix lufs={-14} ceiling={-1.5} />
    </AbsoluteFill>
  );
}

registerRoot(() => (
  <Composition id="edit" component={Pong} width={CW} height={CH} fps={120} durationInFrames={4668} />
));
