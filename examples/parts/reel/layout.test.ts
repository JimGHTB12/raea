// <Ad>'s parts in time (src/layout.ts, the layout the render and the feed's host share): a part that says its lines
// (Phrases) and has no cue of its own starts where its first line is said, as `say` with those words would start it.
// Before, it started at frame 0 and its lines were said outside it, so the render refused the part.
import { describe, expect, test } from "bun:test";
import { firstWords, layout, PARTS, spanOf } from "./layout";
import { findPhrase } from "raea";

// a made-up talking take: one word every 10 frames
const said = "pick a stone grow some moss then name your rock and keep it close follow along";
const words = said.split(" ").map((text, n) => ({ text, from: n * 10, to: n * 10 + 8 }));
const span = (cue: Record<string, unknown>) => spanOf(findPhrase, "Phrases", cue, PARTS.Phrases, words, [], 30, 200);

describe("a Phrases with no cue", () => {
  test("starts where its first line's first words are said, as say= with them does", () => {
    expect(span({ text: "then *name* / your rock / and keep it" })).toEqual(span({ say: "then name", text: "then *name* / your rock / and keep it" }));
    expect(span({ text: "then *name* / your rock" }).from).toBe(60);
  });

  test("ends on its until, said after it starts", () => {
    expect(span({ text: "name your / rock", until: "keep it" })).toEqual({ from: 70, to: 128 });
  });

  test("a cue of its own wins: from, beat or say", () => {
    expect(span({ from: 5, text: "then name" }).from).toBe(5);
    expect(span({ say: "follow along", text: "then name" }).from).toBe(140);
  });

  test("starts at 0 when its first words aren't said (the part then says which line isn't)", () => {
    expect(span({ text: "never said here" }).from).toBe(0);
  });

  test("holds the band from its first line until the next part", () => {
    const laid = layout(findPhrase, [
      { i: 0, name: "Hook", cue: { say: "pick a stone", until: "some moss" } },
      { i: 1, name: "Phrases", cue: { text: "then *name* / your rock / and keep it" } },
      { i: 2, name: "Follow", cue: { say: "follow along" } },
    ], words, [], 30, 200);
    expect(laid.spans.get(1)).toEqual({ from: 60, to: 140 });
  });

  test("its first words: two, or the first alone when it's three letters or more, accents plain", () => {
    expect(firstWords("*You* need more / bets")).toEqual(["You need", "You"]);
    expect(firstWords("a bet")).toEqual(["a bet"]);
    expect(firstWords("Bets")).toEqual(["Bets"]);
  });

  test("only a part that says its lines starts on them: a Hook's text doesn't place it", () => {
    expect(spanOf(findPhrase, "Hook", { text: "then name" }, PARTS.Hook, words, [], 30, 200).from).toBe(0);
  });
});
