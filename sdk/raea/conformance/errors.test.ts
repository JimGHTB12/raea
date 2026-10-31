// What a writer is told when a composition doesn't run, so the next try can fix it (the feed's write loop hands the
// error back as is): a phrase that isn't said names the closest one that is, and a keyframe that isn't a number shows
// the call it's in.
import { describe, expect, test } from "bun:test";
import { findPhrase } from "../../video/src/phrase";
import { interpolate } from "../../math/src/interpolate";

// a made-up talking take: one word every 10 frames
const said = "most teams ship more pages than ever and still the one that works is hard to find";
const words = said.split(" ").map((text, n) => ({ text, from: n * 10, to: n * 10 + 8 }));
const missed = (phrase: string) => {
  try {
    findPhrase(words, phrase);
  } catch (e) {
    return (e as Error).message;
  }
  throw new Error(`"${phrase}" was found`);
};

describe("a phrase that isn't said", () => {
  test("names the closest stretch that is, in the speaker's words", () => {
    expect(missed("more pages than before")).toBe('word("more pages than before"): the phrase is never said on the timeline; the closest that is said: "more pages than ever"');
    expect(missed("the one that sells")).toContain('the closest that is said: "the one that works"');
  });
  test("names none when nothing shares half its words", () => {
    expect(missed("a quiet harbour at dawn")).toBe('word("a quiet harbour at dawn"): the phrase is never said on the timeline');
  });
  test("says when there are no words at all", () => {
    expect(() => findPhrase([], "anything")).toThrow("the timeline has no words");
  });
  test("still counts occurrences", () => {
    expect(() => findPhrase(words, "the one", { occurrence: 2 })).toThrow('word("the one"): the phrase is said only 1 time, not 2, on the timeline');
  });
});

describe("a keyframe that isn't a number", () => {
  test("shows the call's keyframes and the usual cause", () => {
    const missing = {} as { to?: number };
    expect(() => interpolate(10, [0, 30], [0, missing.to as number])).toThrow(
      "interpolate: output[1] is not a number (undefined) in interpolate(…, [0, 30], [0, undefined]): a keyframe came from a missing prop or field",
    );
    expect(() => interpolate(10, [0, 30], [0, NaN])).toThrow("output[1] is not a number (NaN) in interpolate(…, [0, 30], [0, NaN])");
  });
});
