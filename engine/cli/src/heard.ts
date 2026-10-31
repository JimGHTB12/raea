// A take heard: parakeet-mlx's tokens joined into words, the one place it's done. `raea cut` (native.ts) and the feed's
// host (apps/app/host/take.ts) both import it.

/** A word heard, in the take's own seconds */
export type Heard = { text: string; start: number; end: number };

/** Parakeet's tokens as words: a token starting with a space starts one, and so does a digit after a letter
 *  ("just" "2%" come as "just" + "2%" with no space). */
export function wordsOf(tokens: Heard[]): Heard[] {
  const out: Heard[] = [];
  for (const t of tokens) {
    const last = out[out.length - 1];
    const starts = t.text.startsWith(" ") || !last || (/^\d/.test(t.text) && /[a-z]$/i.test(last.text));
    if (starts) {
      if (t.text.trim()) out.push({ text: t.text.trim(), start: t.start, end: t.end });
    } else {
      last.text += t.text;
      last.end = t.end;
    }
  }
  return out;
}
