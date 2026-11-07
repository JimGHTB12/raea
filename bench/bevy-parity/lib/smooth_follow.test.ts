// lib/smooth_follow.ts against bevy_math 0.20's own run of smooth_follow.rs's systems (a scratch crate on bevy_math,
// rand 0.10.3 and chacha20 0.10.2): target, follower and the point it heads for, as f32 bits, at the frames probed.
import { expect, test } from "bun:test";
import { smoothFollow } from "./smooth_follow";

const bits = (x: number) => new Uint32Array(new Float32Array([x]).buffer)[0]!.toString(16).padStart(8, "0");
const TRUTH: Record<number, string> = {
  1: "3d52f770 bd155d9a bd5eea76 3add526d bff7c2a6 bae9db9a 3fe0971c bf9f02e4 bfed4fac",
  10: "3f03daa6 bebab4ff bf0b5289 3dacaa0c bfbf128a bdb671af 3fe0971c bf9f02e4 bfed4fac",
  30: "3fc5c7fb bf8c07c1 bfd0fbd0 3f15a9de bf93288f bf1e23fe 3fe0971c bf9f02e4 bfed4fac",
  60: "3e238854 be540699 bf9deedd 3f4e07ea bf5eefa2 bf98e54a bfda49d4 3f807360 bf0237b0",
  90: "bfda49d4 3f807360 bf0237b0 beb3c37b 3d0ca65c bf6db132 bff66ebc 3fe101ec bf700c38",
  120: "bf55f53e 3f3ed6e3 bf90ffa1 bf86169e 3f4f3053 bf74ccb4 be807650 3e4ed200 bf9e67e8",
  180: "be967d1e 3f35c1b6 bc528d20 be611fd8 3f01e743 3d52d8c0 bfa6ca70 3f520018 bfc64714",
};

test("smoothFollow is bevy_math's run, bit for bit", () => {
  for (const [frame, want] of Object.entries(TRUTH)) expect(`${frame} ${smoothFollow(Number(frame)).flat().map(bits).join(" ")}`).toBe(`${frame} ${want}`);
});
