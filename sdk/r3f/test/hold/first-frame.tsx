// An <R3F> scene that suspends on its first frame, as drei's <Environment> does while its HDR loads: the box's colour
// comes from a promise that resolves 50 ms later. A render must wait for it (hold.tsx): frame 0 is the red box, not
// the 2D background alone.
import { use } from "react";
import { AbsoluteFill, Composition, registerRoot } from "raea";
import { R3F } from "raea/r3f";

const COLOUR = new Promise<string>((ok) => setTimeout(() => ok("red"), 50));

function Box() {
  const color = use(COLOUR);
  return (
    <mesh>
      <boxGeometry args={[1.5, 1.5, 1.5]} />
      <meshBasicMaterial color={color} />
    </mesh>
  );
}

function Scene() {
  return (
    <AbsoluteFill style={{ background: "#224466" }}>
      <R3F camera={{ position: [0, 0, 3] }}>
        <Box />
      </R3F>
    </AbsoluteFill>
  );
}
registerRoot(() => <Composition id="hold" component={Scene} width={160} height={160} fps={30} durationInFrames={3} />);
