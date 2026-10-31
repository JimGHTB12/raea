// findPhrase: where a phrase is said in a transcript, with no React and no imports, so a tool off the render (the
// feed's host, laying a composition's parts out on its words) finds a phrase exactly as word() does. words.tsx
// re-exports it.

export interface WordOptions {
  /** which time the phrase is said, from 1 */
  occurrence?: number;
  /** "start": the phrase's first frame; "end": the frame after its last word ends */
  edge?: "start" | "end";
}

/** a word heard on the timeline: its first frame, and the frame after it ends */
export interface HeardWord {
  text: string;
  from: number;
  to: number;
}

/** Matching ignores case and punctuation: only letters and digits count. */
function normalise(text: string): string {
  return text.toLowerCase().replace(/[^\p{L}\p{N}]+/gu, "");
}

function tokens(phrase: string): string[] {
  return phrase.split(/\s+/).map(normalise).filter((t) => t !== "");
}

/** Finds the composition frame where `phrase` is said in a transcript. */
export function findPhrase(words: readonly HeardWord[], phrase: string, options: WordOptions = {}): number {
  const { occurrence = 1, edge = "start" } = options;
  if (!Number.isInteger(occurrence) || occurrence < 1) {
    throw new RangeError(`word("${phrase}"): occurrence must be a whole number from 1, got ${occurrence}`);
  }
  if (edge !== "start" && edge !== "end") throw new RangeError(`word("${phrase}"): edge must be "start" or "end"`);
  const wanted = tokens(phrase);
  if (wanted.length === 0) throw new Error(`word("${phrase}"): the phrase has no words to match`);

  // the transcript as matchable tokens, skipping entries that are only punctuation
  const heard = words.map((w) => ({ key: normalise(w.text), word: w })).filter((h) => h.key !== "");
  let found = 0;
  for (let i = 0; i + wanted.length <= heard.length; i++) {
    let ok = true;
    for (let j = 0; j < wanted.length; j++) {
      if (heard[i + j].key !== wanted[j]) {
        ok = false;
        break;
      }
    }
    if (ok && ++found === occurrence) {
      return edge === "start" ? heard[i].word.from : heard[i + wanted.length - 1].word.to;
    }
  }
  if (heard.length === 0) throw new Error(`word("${phrase}"): the timeline has no words (no transcript was given)`);
  if (found > 0) throw new Error(`word("${phrase}"): the phrase is said only ${found} time${found === 1 ? "" : "s"}, not ${occurrence}, on the timeline`);
  const near = closest(heard, wanted);
  throw new Error(`word("${phrase}"): the phrase is never said on the timeline${near ? `; the closest that is said: "${near}"` : ""}`);
}

/** The stretch of the transcript, as long as the phrase, sharing the most of its words (at least half; on a tie, the
 *  one with more in the same place), as said: a writer who paraphrased gets the speaker's own words to copy */
function closest(heard: readonly { key: string; word: HeardWord }[], wanted: string[]): string | undefined {
  const n = Math.min(wanted.length, heard.length);
  let best = 0;
  let placed = 0;
  let at = -1;
  for (let i = 0; i + n <= heard.length; i++) {
    const left = [...wanted];
    let shared = 0;
    for (let j = 0; j < n; j++) {
      const k = left.indexOf(heard[i + j].key);
      if (k >= 0) {
        left.splice(k, 1);
        shared++;
      }
    }
    let same = 0;
    for (let j = 0; j < n; j++) if (heard[i + j].key === wanted[j]) same++;
    if (shared > best || (shared === best && same > placed)) [best, placed, at] = [shared, same, i];
  }
  if (at < 0 || best * 2 < wanted.length) return undefined;
  return heard.slice(at, at + n).map((h) => h.word.text).join(" ");
}
