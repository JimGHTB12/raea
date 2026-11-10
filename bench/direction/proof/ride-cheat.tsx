// DR4's proof, the 3D scene's cheat: the rider turned with the seat (so a check of the angle alone passes), but pushed
// back into the seat's back (seat.tsx). checks/rider.ts must fail it.
import { Composition, registerRoot, useCurrentFrame } from "raea";
import { Seat } from "./seat";

function Ride() {
  return <Seat yaw={0.3 + useCurrentFrame() * 0.01} rider={[0, 0.35, -0.15]} />;
}

registerRoot(() => <Composition id="ride-cheat" component={Ride} width={540} height={960} fps={30} durationInFrames={30} />);
