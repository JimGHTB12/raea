// word() and useWords(): time anchored to what's said (spec: "Time anchored to words").
import { useClockState, useLocalTime, type TimelineWord } from "@raea/app/clock";
import { findPhrase, type WordOptions } from "./phrase";

export { findPhrase, type WordOptions } from "./phrase";

/** The composition frame where `phrase` is said. Call it while rendering (it reads the Clock). */
export function word(phrase: string, options?: WordOptions): number {
  return findPhrase(useClockState("word").words, phrase, options);
}

/** Every word heard on the timeline, in the frames of the nearest enclosing Sequence. */
export function useWords(): TimelineWord[] {
  const clock = useClockState("useWords");
  const { origin } = useLocalTime("useWords");
  return clock.words.map((w) => ({ text: w.text, from: w.from - origin, to: w.to - origin }));
}
