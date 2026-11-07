// lib/rand.ts against chacha20 0.10.2 and rand 0.10.3's own output (a scratch crate printing ChaCha8Rng::seed_from_u64(
// 19878367467713), spotlight.rs's seed: its first words, the words after the first refill of its 64-word buffer, and
// random_range(-5.0..5.0) as f32 bits).
import { expect, test } from "bun:test";
import { ChaCha8Rng } from "./rand";

const bits = (x: number) => new Uint32Array(new Float32Array([x]).buffer)[0]!.toString(16).padStart(8, "0");

test("ChaCha8Rng's words and random_range are the crates' own", () => {
  const r = ChaCha8Rng.seed_from_u64(19878367467713n);
  const w = Array.from({ length: 70 }, () => r.next_u32());
  expect(w.slice(0, 4)).toEqual([2045446710, 148267730, 2984187155, 4078822190]);
  expect(w.slice(64, 70)).toEqual([3322331382, 1283986520, 4195170605, 2259972292, 328924921, 2056473586]);
  const g = ChaCha8Rng.seed_from_u64(19878367467713n);
  expect(Array.from({ length: 6 }, () => bits(g.random_range(-5, 5)))).toEqual(["be734680", "c094f406", "3ff95b64", "408fe55a", "c0396a22", "bf7365c0"]);
});

test("random_dir3 is bevy_math's random::<Dir3>() (align.rs's seed)", () => {
  const r = ChaCha8Rng.seed_from_u64(19878367467712n);
  expect([r.random_dir3().map(bits), r.random_dir3().map(bits)]).toEqual([["be848688", "3e9c2559", "3f6aa088"], ["bf2dc30a", "beef6b4a", "bf10f4e8"]]);
});
