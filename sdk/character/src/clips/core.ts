// Tags, the file's layout, mirroring and HDC encoding.
import { FEATURE_COUNT as NF, STATE } from "../matching";

// ---------------------------------------------------------------- tags

/** The tag vocabulary (at most 64: two words a frame). The first are read from a capture's description, the last
 * measured on each frame. */
export const TAGS = [
  "walk", "run", "jog", "turn", "jump", "hop", "sit", "stand", "idle", "carry", "suitcase", "box", "heavy", "pick", "place", "reach",
  "look-around", "climb", "stairs", "crouch", "sneak", "dance", "fight", "throw", "catch", "drink", "chore", "push", "pull", "balance", "hurt", "styled",
  "backward", "sideways", "uneven", "playground", "swing", "stretch", "lie", "fall", "sport", "gesture", "open", "two-person", "100style", "fw", "id", "cmu",
  "moving", "still", "fast", "crouched", "raised",
] as const;

export type Tag = (typeof TAGS)[number];

export const TAG_INDEX = new Map<string, number>(TAGS.map((t, i) => [t, i]));

const WORDS: [Tag, RegExp][] = [
  ["walk", /walk|stroll|wander|march|stride|stepping|navigate|veer|hobble|mope|quail|footfalls/],
  ["run", /\brun|running|sprint|scramble/],
  ["jog", /jog/],
  ["turn", /turn|spin|veer|pivot|figure ?8|zigzag|circle/],
  ["jump", /jump|leap/],
  ["hop", /hop(?!scotch)|hopscotch/],
  ["sit", /\bsit|seat|chair|sitting/],
  ["stand", /stand|get up|rise/],
  ["idle", /idle|wait|still|yawn|fidget|hang out|stares|bored/],
  ["carry", /carry|carries|moving .*box|laundry basket|coffee mug|hold/],
  ["suitcase", /suitcase|luggage/],
  ["box", /\bbox\b|boxes|basket/],
  ["heavy", /heavy|\d+(\.\d+)? ?lb/],
  ["pick", /pick|grab|scoop|lift/],
  ["place", /place|put down|set down|\bput\b/],
  ["reach", /reach/],
  ["look-around", /look|search|blind man/],
  ["climb", /climb|ladder|ledge|pull up|descend|step up|stepping up/],
  ["stairs", /stair|step ?stool|steps\b|step up|stepping up/],
  ["crouch", /crouch|squat|bend|bent|duck|knees bent|low\b/],
  ["sneak", /sneak|creep|tiptoe|stealth/],
  ["dance", /nursery rhyme|teapot|danc|salsa|charleston|waltz|ballet|lindy|twist|tap\b|swing dance|mickey|singing in the rain/],
  ["fight", /punch|kick|boxing|fight|martial|karate|attack|sword|wrestl|mawashigeri|gun|smash/],
  ["throw", /throw|toss|pitch/],
  ["catch", /catch/],
  ["drink", /drink|soda|\beat/],
  ["chore", /wash|sweep|wipe|mop|chop|paint|broom/],
  ["push", /push/],
  ["pull", /pull/],
  ["balance", /balanc|beam|arms out/],
  ["hurt", /limp|hurt|wound|injur|drunk|stagger|stumble|achey|drag bad|spastic/],
  ["styled", /superhero|old man|sexy|happy|sad|angr|proud|cat |chicken|dinosaur|gangly|graceful|stiff|bouncy|zombie|robot|mime|frankenstein|mummy|macho|ghetto|cool|attitude|stern|shy|scared|lavish|excited|depressed|clumsy|exaggerated|wild|pigeon|duck footed|cross ?over|jaunt|muscular|digital|alien|lady|strong man|teen|soldiers/],
  ["backward", /backward/],
  ["sideways", /sideway|sidestep|side step/],
  ["uneven", /uneven|terrain|rough|slope/],
  ["playground", /playground|monkey bar|jungle gym|dangle/],
  ["swing", /swing/],
  ["stretch", /stretch|exercise|yoga/],
  ["lie", /\blay|lying|\blie\b|roll|laying|crawl|face down|from floor|from ground/],
  ["fall", /fall/],
  ["sport", /golf|basketball|soccer|football|baseball|tennis|dribble|drive from|defensive|free throw/],
  ["gesture", /wave|gesture|point|clap|signal|explain|high-five|low-five|shake hands|whistle/],
  ["open", /open|door|window|umbrella|unscrew|keys|wallet|bottlecap/],
  ["two-person", /2 subjects/],
];

/** A capture's tags from its description (CMU's free text; a 100STYLE clip's label and move). Pure. */
export function tagsOf(label: string, source = "cmu", move?: string): Tag[] {
  const out = new Set<Tag>();
  if (source === "100style") {
    out.add("100style");
    out.add(move === "ID" ? "id" : "fw");
    out.add(move === "ID" ? "idle" : "walk");
    if (label !== "Neutral") out.add("styled");
    return [...out];
  }
  out.add("cmu");
  // (CamelCase run together, "CarefulWalk LookingAround", split into words)
  const l = label.replace(/([a-z])([A-Z])/g, "$1 $2").replace(/_/g, " ").toLowerCase();
  for (const [t, re] of WORDS) if (re.test(l)) out.add(t);
  return [...out];
}

// ---------------------------------------------------------------- the file

/** quantisation: a normalised feature or a state z-score in 1/QF of a unit */
export const QF = 128;

export const NAN16 = -32768;

/** per-frame flag bits */
export const FLAG = { route: 1, contactL: 2, contactR: 4, crouch: 8, moving: 16, ok: 32 } as const;

/** the HDC: 8 words (256 bits) a frame, 16 levels of half a unit over +-4 */
export const HW = 8, HLEVELS = 16, HSTEP = 64;

/** a frame's row strides in the file: features and state padded to whole words */
export const FS = 28, SS = 22;

const MAGIC = "RAEAMDB1";

export interface DbClip { name: string; source: string; file: string; label: string; move?: string; tags: Tag[]; start: number; frames: number }

export interface DbHeader {
  /** 2: with the hands (U); 3: and the hands' swing clearances (K) */
  version: 1 | 2 | 3;
  skeleton: string;
  fps: number;
  /** unmirrored frames */
  frames: number;
  nf: number;
  ns: number;
  qf: number;
  mean: number[];
  scale: number[];
  stateMean: number[];
  stateSd: number[];
  tags: string[];
  hdc: { words: number; levels: number; step: number };
  /** a mirror image's columns: feature d is f[d] of the frame times fs[d], state i is s[i] times ss[i] */
  mirror: { f: number[]; fs: number[]; s: number[]; ss: number[] };
  clips: DbClip[];
  sections: Record<string, { offset: number; bytes: number }>;
}

export interface MotionDb {
  header: DbHeader;
  /** frames x FS i16: the normalised features x QF (NAN16 where the clip doesn't run on a second) */
  F: Int16Array;
  /** frames x SS i16: the STATE z-scores x QF */
  S: Int16Array;
  /** frames x 3: ground x, z, yaw */
  G: Float32Array;
  flags: Uint8Array;
  /** the hips' height over the clip's median, x1000 */
  hip: Uint16Array;
  /** the hips' ground speed over the second about the frame, cm/s */
  speed: Uint16Array;
  /** frames x 2 words of tag bits */
  T: Uint32Array;
  /** frames x HW words: the HDC hypervector */
  H: Uint32Array;
  /** frames x 6 i16, mm: the left hand's and the right's place in the frame's ground frame (x to the body's left, y up
   * from the floor, z ahead), for an upper body's search (per-limb control); absent in databases built before it */
  U?: Int16Array;
  /** frames x 2 i16, mm: the left hand's and the right's swing clearance (matching.ts swingClear: a thing held in it
   * to the skin of its thigh and of the waist), for keeping a thing held clear of the legs; absent before version 3 */
  K?: Int16Array;
}

const SECTIONS = ["F", "S", "G", "flags", "hip", "speed", "T", "H"] as const;

/** The file: magic, the header's length (u32) and a pad to 16 bytes, the JSON header, then each section 64-byte aligned. */
export function writeDb(db: Omit<MotionDb, "header"> & { header: Omit<DbHeader, "sections"> }): Uint8Array {
  const parts: Record<string, Uint8Array> = {};
  const secs: string[] = [...SECTIONS, ...(db.U ? ["U"] : []), ...(db.K ? ["K"] : [])];
  for (const s of secs) { const a = (db as any)[s] as ArrayBufferView; parts[s] = new Uint8Array(a.buffer, a.byteOffset, a.byteLength); }
  // the header's length depends on the offsets, the offsets on its length: lay out with room to spare
  const draft = (sections: DbHeader["sections"]) => new TextEncoder().encode(JSON.stringify({ ...db.header, sections }));
  let sections: DbHeader["sections"] = {}, head = draft(Object.fromEntries(secs.map((s) => [s, { offset: 1e12, bytes: 1e12 }])));
  const base = Math.ceil((16 + head.length + 64) / 64) * 64;
  let at = base;
  for (const s of secs) { sections[s] = { offset: at, bytes: parts[s].length }; at = Math.ceil((at + parts[s].length) / 64) * 64; }
  head = draft(sections);
  if (16 + head.length > base) throw new Error("writeDb: the header outgrew its room");
  const out = new Uint8Array(at);
  out.set(new TextEncoder().encode(MAGIC), 0);
  new DataView(out.buffer).setUint32(8, head.length, true);
  out.set(head, 16);
  out.fill(0x20, 16 + head.length, base);
  for (const s of secs) out.set(parts[s], sections[s].offset);
  return out;
}

/** A database from its bytes (clips-bun.ts readDb reads the file) */
export function parseDb(bytes: Uint8Array): MotionDb {
  if (new TextDecoder().decode(bytes.subarray(0, 8)) !== MAGIC) throw new Error("readDb: not a motion database");
  const len = new DataView(bytes.buffer, bytes.byteOffset).getUint32(8, true);
  const header = JSON.parse(new TextDecoder().decode(bytes.subarray(16, 16 + len))) as DbHeader;
  const view = <T>(s: string, C: { new (b: ArrayBuffer, o: number, n: number): T; BYTES_PER_ELEMENT: number }): T => {
    const x = header.sections[s];
    return new C(bytes.buffer as ArrayBuffer, bytes.byteOffset + x.offset, x.bytes / C.BYTES_PER_ELEMENT);
  };
  return { header, F: view("F", Int16Array), S: view("S", Int16Array), G: view("G", Float32Array), flags: view("flags", Uint8Array), hip: view("hip", Uint16Array), speed: view("speed", Uint16Array), T: view("T", Uint32Array), H: view("H", Uint32Array), ...(header.sections.U ? { U: view("U", Int16Array) } : {}), ...(header.sections.K ? { K: view("K", Int16Array) } : {}) };
}

// ---------------------------------------------------------------- mirroring

/** feature d of a mirror image: feature MF_SRC[d] of the frame, times MF_SIGN[d] (the feet swapped, x and yaws negated:
 * matching.ts mirrorRow) */
export const MF_SRC = Array.from({ length: NF }, (_, d) => { if (d >= 12) return d; const g = d < 6 ? 0 : 6, a = (d - g) % 3, side = Math.floor((d - g) / 3); return g + (1 - side) * 3 + a; });

export const MF_SIGN = Array.from({ length: NF }, (_, d) => ([0, 3, 6, 9, 12, 15, 17, 19, 21, 23, 25].includes(d) ? -1 : 1));

export const MS_SRC = STATE.map((k) => (k === "armSwingL" ? STATE.indexOf("armSwingR") : k === "armSwingR" ? STATE.indexOf("armSwingL") : STATE.indexOf(k)));

export const MS_SIGN = STATE.map((k) => (k === "trunkRoll" || k === "asymSwing" || k === "asymStance" ? -1 : 1));

/** round half away from zero, so a value and its negation quantise to negations (a mirror stays exact) */
export const qround = (x: number) => (x < 0 ? -Math.round(-x) : Math.round(x));

export const q16 = (x: number) => (Number.isNaN(x) ? NAN16 : Math.max(-32767, Math.min(32767, qround(x * QF))));

// ---------------------------------------------------------------- HDC

/** look_proj.k's integer hash: a random bit for (a, salt), the same in TS, the K interpreter and WGSL */
export function hbit(a: number, salt: number): number {
  const h = Math.imul((Math.imul(a, 73856093) ^ Math.imul(salt, 19349663)) + 40503 | 0, -1640531535);
  return (h >> 15) & 1;
}

/** a feature's level: its value in HSTEP steps from -4 units, 0..HLEVELS-1 */
export const hlevel = (q: number) => Math.max(0, Math.min(HLEVELS - 1, (q + 512) >> 6));

/** a random threshold 0..15 for (a): four hash bits */
const hthr = (a: number) => hbit(a, 2) | (hbit(a, 3) << 1) | (hbit(a, 4) << 2) | (hbit(a, 5) << 3);

/** HDC role and threshold tables: bit g of feature d's role r(d, g), and its flip threshold; bit g of tag t's role */
const RT = (() => {
  const B = HW * 32, role = new Uint8Array(NF * B), thr = new Uint8Array(NF * B), tag = new Uint8Array(64 * B), tie = new Uint8Array(B);
  for (let d = 0; d < NF; d++) for (let g = 0; g < B; g++) { role[d * B + g] = hbit(d * B + g, 1); thr[d * B + g] = hthr(d * B + g); }
  for (let t = 0; t < 64; t++) for (let g = 0; g < B; g++) tag[t * B + g] = hbit((64 + t) * B + g, 1);
  for (let g = 0; g < B; g++) tie[g] = hbit(255 * B + g, 1);
  return { role, thr, tag, tie };
})();

/** A hypervector (HW words into out at o) of quantised features q (NF, from qo) bundled with tags (two words; none:
 * the motion alone): bit g the majority of each feature's role bit flipped where its level passes the bit's threshold
 * (Levels bound to roles), and each present tag's role bit; a tie the tie vector's. */
export function hdcEncode(q: ArrayLike<number>, qo: number, t0: number, t1: number, out: Uint32Array, o: number, withTags = true) {
  const B = HW * 32, lv = new Int32Array(NF), tg: number[] = [];
  for (let d = 0; d < NF; d++) lv[d] = q[qo + d] === NAN16 ? -1 : hlevel(q[qo + d]);
  if (withTags) for (let t = 0; t < 64; t++) if (((t < 32 ? t0 >>> t : t1 >>> (t - 32)) & 1)) tg.push(t);
  let total = 0;
  for (let d = 0; d < NF; d++) if (lv[d] >= 0) total++;
  total += tg.length;
  for (let w = 0; w < HW; w++) {
    let word = 0;
    for (let b = 0; b < 32; b++) {
      const g = w * 32 + b;
      let c = 0;
      for (let d = 0; d < NF; d++) if (lv[d] >= 0) c += RT.role[d * B + g] ^ (RT.thr[d * B + g] < lv[d] ? 1 : 0);
      for (const t of tg) c += RT.tag[t * B + g];
      const bit = 2 * c > total ? 1 : 2 * c === total ? RT.tie[g] : 0;
      word |= bit << b;
    }
    out[o + w] = word >>> 0;
  }
}

export const popcount = (x: number) => { x -= (x >>> 1) & 0x55555555; x = (x & 0x33333333) + ((x >>> 2) & 0x33333333); return (Math.imul((x + (x >>> 4)) & 0x0f0f0f0f, 0x01010101) >>> 24); };
