// #18: what an element paints at an offset of its own (background-position, a background's tiles, a box shadow) moves
// in the element's axes, so it turns with a rotated ancestor. Each composition draws the same picture two ways under a
// -4deg rotation: "sliced" splits one background image and one gradient across stacked divs by background-position,
// and gives a box a shadow; "whole" draws each image in one div and the shadow as a box behind. check.sh compares them.
import { AbsoluteFill, Composition, registerRoot } from "raea";

// an 8 x 8 of four quarters (yellow, clear, blue, green), drawn 240 px a side
const QUAD = "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAgAAAAICAYAAADED76LAAAAIUlEQVR4nGP4dUb0PzJmQAd0UKCa/Po/MlY6GoeC6aAAABvgeQHBIU/gAAAAAElFTkSuQmCC";
const GRAD = "repeating-linear-gradient(35deg, #e11d48 0 18px, #0ea5e9 18px 36px, #fde047 36px 54px)";
const W = 240;
const H = 300;
const CUTS = [0, 110, 190, H];
const at = (left: number, top: number, w: number, h: number) => ({ position: "absolute", left, top, width: w, height: h }) as const;
const image = { backgroundImage: `url("${QUAD}")`, backgroundSize: `${W}px ${W}px`, backgroundRepeat: "repeat", imageRendering: "pixelated" } as const;

function Stack({ sliced, left, bg }: { sliced: boolean; left: number; bg: object }) {
  if (!sliced) return <div style={{ ...at(left, 0, W, H), ...bg }} />;
  return (
    <>
      {CUTS.slice(1).map((end, k) => (
        <div key={k} style={{ ...at(left, CUTS[k], W, end - CUTS[k]), ...bg, backgroundPosition: `0px ${-CUTS[k]}px` }} />
      ))}
    </>
  );
}

function Picture({ sliced }: { sliced: boolean }) {
  return (
    <AbsoluteFill style={{ background: "#151515" }}>
      <div style={{ ...at(200, 150, 900, 420), transform: "rotate(-4deg)", transformOrigin: "50% 60%" }}>
        <Stack sliced={sliced} left={0} bg={image} />
        <Stack sliced={sliced} left={300} bg={{ backgroundImage: GRAD, backgroundSize: `${W}px ${H}px` }} />
        {!sliced && <div style={{ ...at(620, 60, 200, 160), background: "#a3e635" }} />}
        <div style={{ ...at(600, 30, 200, 160), background: "#f6f1e8", boxShadow: sliced ? "20px 30px 0 #a3e635" : undefined }} />
      </div>
    </AbsoluteFill>
  );
}

registerRoot(() => (
  <>
    <Composition id="sliced" component={() => <Picture sliced />} width={1280} height={720} fps={30} durationInFrames={1} />
    <Composition id="whole" component={() => <Picture sliced={false} />} width={1280} height={720} fps={30} durationInFrames={1} />
  </>
));
