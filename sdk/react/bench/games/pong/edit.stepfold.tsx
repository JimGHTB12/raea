// pong with every hook kept and nothing drawn: R1/R2's step+fold-alone variant (bench/lightspeed/jsheap.sh, lightspeed Z3)
// PONG — a table-tennis table seen from directly overhead in an empty night hall,
// turned sideways into a classic court: you play the left paddle, the computer the right.
// The game is a pure fold: live arrow keys when someone plays, the scripted attract
// match (input.ts) when nobody does. The court is drawn in its own units and scaled
// to fill whatever stage it is given.
import {
  Audio, Composition, Sequence, registerRoot, useCurrentFrame, useGameEvents, useGameState, useInput, useVideoConfig,
} from "raea";
import { AbsoluteFill } from "raea";
import { css } from "./styles.gen";
import { INPUT } from "./input";
import { CW, CH, TARGET, INITIAL, EMPTY, fold, type Mark } from "./game";
import { POCK, TOCK, THOCK, TAK, CLACK, SCUFF, BELL, HALL } from "./sounds";

function Sounds({
  hits, walls, tables, points, over, steps,
}: {
  hits: number[]; walls: number[]; tables: number[]; points: number[]; over: number[]; steps: number[];
}) {
  return (
    <>
      <Sequence from={0} layout="none">
        <Audio src={HALL} gainDb={-22} fadeIn={1.4} fadeOut={2.6} />
      </Sequence>
      {hits.map((f, i) => (
        <Sequence key={"h" + i} from={f} layout="none">
          <Audio src={i % 2 ? TOCK : POCK} gainDb={i % 2 ? -6 : -4} fadeIn={0.012} fadeOut={0.08} />
        </Sequence>
      ))}
      {walls.map((f, i) => (
        <Sequence key={"w" + i} from={f} layout="none">
          <Audio src={THOCK} gainDb={-9} fadeIn={0.012} fadeOut={0.08} />
        </Sequence>
      ))}
      {tables.map((f, i) => (
        <Sequence key={"t" + i} from={f} layout="none">
          <Audio src={TAK} gainDb={-9} fadeIn={0.012} fadeOut={0.08} />
        </Sequence>
      ))}
      {steps.map((f, i) => (
        <Sequence key={"s" + i} from={f} layout="none">
          <Audio src={SCUFF} gainDb={-14} fadeIn={0.012} fadeOut={0.06} />
        </Sequence>
      ))}
      {points.map((f, i) => (
        <Sequence key={"p" + i} from={f} layout="none">
          <Audio src={CLACK} gainDb={-5} fadeIn={0.012} fadeOut={0.12} />
        </Sequence>
      ))}
      {over.map((f, i) => (
        <Sequence key={"o" + i} from={f} layout="none">
          <Audio src={BELL} gainDb={-7} fadeIn={0.012} fadeOut={0.4} />
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

  void frame; void stage; void inp; void g; void hits; void walls; void tables; void points; void over;
  return null;
}

registerRoot(() => (
  <Composition id="edit" component={Pong} width={CW} height={CH} fps={120} durationInFrames={4668} />
));
