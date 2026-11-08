// The random numbers Bevy's examples draw (30 of them seed a ChaCha8Rng): chacha20 0.10's ChaCha8Rng, seeded by
// rand_core 0.10's seed_from_u64 (PCG32 filling the 32-byte key), and rand 0.10's random_range over an f32 range (23
// random bits as a float in [1, 2), less 1, times the span, plus the low end, in f32). Checked against the crates'
// own output (bench/bevy-parity/lib/rand.test.ts).

const M64 = (1n << 64n) - 1n;

/** rand_core's seed_from_u64: eight PCG32 outputs, the key's words */
function seedWords(seed: bigint): number[] {
  let state = seed & M64;
  const words: number[] = [];
  for (let i = 0; i < 8; i++) {
    state = (state * 0x5851f42d4c957f2dn + 0xa17654e46fbe17f3n) & M64;
    const xorshifted = Number((((state >> 18n) ^ state) >> 27n) & 0xffffffffn);
    const rot = Number(state >> 59n);
    words.push(((xorshifted >>> rot) | (xorshifted << ((32 - rot) & 31))) >>> 0);
  }
  return words;
}

const rotl = (x: number, n: number) => ((x << n) | (x >>> (32 - n))) >>> 0;

/** chacha20's ChaCha8Rng: the key from the seed, a 64-bit block counter, stream 0, four blocks buffered at a time */
export class ChaCha8Rng {
  private state: number[];
  private buf: number[] = [];
  private at = 64;
  private constructor(key: number[]) {
    this.state = [0x61707865, 0x3320646e, 0x79622d32, 0x6b206574, ...key, 0, 0, 0, 0];
  }
  static seed_from_u64(seed: number | bigint): ChaCha8Rng {
    return new ChaCha8Rng(seedWords(BigInt(seed)));
  }
  private block(): number[] {
    const x = this.state.slice();
    const qr = (a: number, b: number, c: number, d: number) => {
      x[a] = (x[a]! + x[b]!) >>> 0; x[d] = rotl(x[d]! ^ x[a]!, 16);
      x[c] = (x[c]! + x[d]!) >>> 0; x[b] = rotl(x[b]! ^ x[c]!, 12);
      x[a] = (x[a]! + x[b]!) >>> 0; x[d] = rotl(x[d]! ^ x[a]!, 8);
      x[c] = (x[c]! + x[d]!) >>> 0; x[b] = rotl(x[b]! ^ x[c]!, 7);
    };
    for (let i = 0; i < 4; i++) {
      qr(0, 4, 8, 12); qr(1, 5, 9, 13); qr(2, 6, 10, 14); qr(3, 7, 11, 15);
      qr(0, 5, 10, 15); qr(1, 6, 11, 12); qr(2, 7, 8, 13); qr(3, 4, 9, 14);
    }
    const out = x.map((v, i) => (v + this.state[i]!) >>> 0);
    // the 64-bit block counter, words 12 and 13
    this.state[12] = (this.state[12]! + 1) >>> 0;
    if (this.state[12] === 0) this.state[13] = (this.state[13]! + 1) >>> 0;
    return out;
  }
  private refill() {
    this.buf = [...this.block(), ...this.block(), ...this.block(), ...this.block()];
  }
  next_u32(): number {
    if (this.at >= 64) {
      this.refill();
      this.at = 0;
    }
    return this.buf[this.at++]!;
  }
  /** rand_core's next_u64_from_u32: two words, the low first; at the buffer's last word, the refill's first is the high */
  next_u64(): bigint {
    const at = this.at;
    let lo: number, hi: number;
    if (at < 63) {
      [lo, hi] = [this.buf[at]!, this.buf[at + 1]!];
      this.at = at + 2;
    } else {
      lo = this.buf[63] ?? 0;
      this.refill();
      [hi, this.at] = [this.buf[0]!, 1];
      if (at >= 64) [lo, hi, this.at] = [hi, this.buf[1]!, 2];
    }
    return (BigInt(hi) << 32n) | BigInt(lo);
  }
  /** rand's random_bool(p): Bernoulli, a u64 under p * 2^64 (p = 1 draws nothing) */
  random_bool(p: number): boolean {
    if (p >= 1) return true;
    return this.next_u64() < BigInt(p * 2 ** 64);
  }
  /** rand's random_range(..len) for a usize under 2^32 (IndexedRandom::choose's draw): a u32 times len, its high word,
   *  one more draw when the low word lands in the biased zone (Lemire's, as rand 0.10 samples a single value) */
  random_index(len: number): number {
    const range = BigInt(len);
    const m = BigInt(this.next_u32()) * range;
    let result = m >> 32n;
    const lo = m & 0xffffffffn;
    if (lo > ((1n << 32n) - range)) {
      const hi2 = (BigInt(this.next_u32()) * range) >> 32n;
      if (lo + hi2 > 0xffffffffn) result += 1n;
    }
    return Number(result);
  }
  /** rand's random_range(low..high) for f32 */
  random_range(low: number, high: number): number {
    const f = Math.fround;
    const bits = (this.next_u32() >>> 9) | 0x3f800000;
    const v = new Float32Array(new Uint32Array([bits]).buffer)[0]!;
    return f(f(f(v - 1) * f(high - low)) + low);
  }
  /** bevy_math's random::<Dir3>() (sample_unit_sphere_boundary): z in [-1, 1], an angle in [-PI, PI] (rand's inclusive
   *  range is the same draw as the half-open one) */
  random_dir3(): [number, number, number] {
    const f = Math.fround;
    const pi = f(Math.PI);
    const z = this.random_range(-1, 1);
    const a = this.random_range(-pi, pi);
    const c = f(Math.sqrt(f(1 - f(z * z))));
    return [f(f(Math.sin(a)) * c), f(f(Math.cos(a)) * c), z];
  }
}
