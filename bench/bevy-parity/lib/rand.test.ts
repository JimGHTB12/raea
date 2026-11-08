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

test("next_u64, random_bool and random_index are the crates' own (gpu_component_array_buffer's seed)", () => {
  const r = ChaCha8Rng.seed_from_u64(12345n);
  const w = Array.from({ length: 63 }, () => r.next_u32());
  expect(w.slice(0, 3)).toEqual([2874758099, 316557125, 2012221028]);
  // the third straddles the refill: the buffer's last word low, the new buffer's first high
  expect(Array.from({ length: 3 }, () => r.next_u64())).toEqual([1152078820691892869n, 8808416481836467466n, 2442021293179131675n]);
  const g = ChaCha8Rng.seed_from_u64(12345n);
  const cubes = Array.from({ length: 5 }, () => {
    const f = [g.random_range(-1, 1), g.random_range(-1, 1), g.random_range(0, 1), g.random_range(0, 1), g.random_range(0, 1)].map(bits);
    return [...f, g.random_bool(0.5)];
  });
  expect(cubes).toEqual([
    ["3ead6548", "bf5a4374", "3eefe01c", "3d2de5e0", "3ed1a320", true],
    ["beaa93b0", "be2e64e0", "3db52ed0", "3f292d96", "3e33d148", false],
    ["bf6edcc4", "be82f000", "3f0466f8", "3ec41798", "3ec3e9f0", false],
    ["bf706d30", "bdcfb8a0", "3f1dc71e", "3e8a4e68", "3d81d6d0", true],
    ["3e4034e0", "3e8730c0", "3f08da06", "3e61b088", "3f4ac6ee", true],
  ]);
  expect(Array.from({ length: 6 }, () => g.random_index(7))).toEqual([4, 4, 4, 4, 2, 1]);
  expect(Array.from({ length: 6 }, () => g.random_index(3))).toEqual([1, 1, 0, 0, 0, 1]);
});
