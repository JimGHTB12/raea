// Strike, Phrases, Callout, Follow and EndCard.
import { useContext } from "react";
import { AbsoluteFill, Img, findPhrase, useCurrentFrame, useWords } from "raea";
import { PARTS } from "./layout";
import { ACCENT, BLIP, BODY, CARD, Cue, Hit, INK, LockLine, ON_ACCENT, OVER, POP, SHADOW, STRIKE, THUD, TOP, TopContext, W, accent, ems, fit, key, landings, lines, part, plain, rise, saidFrom, type } from "./all";

export interface StrikeProps extends Cue {
  /** the lockup above the card, " / " between lines: what's right */
  text: string;
  /** what's wrong, on the card (two lines at most, " / " between them), struck through */
  not: string;
  /** the phrase the strike lands on (a few frames before the part ends when not given) */
  strikeSay?: string;
}

/** "this, not that": the right way big, the wrong way on a card under it, struck through as it's dismissed. */
export const Strike = part<StrikeProps>("Strike", PARTS.Strike, ({ text, not, strikeSay, sound, span }) => {
  const f = useCurrentFrame();
  const TOP = useContext(TopContext);
  const words = useWords();
  const len = span.to - span.from;
  const ls = lines(text).slice(0, 2);
  const rows = lines(not).slice(0, 2);
  // the struck card follows the headline by a third of a second, unless its first row is said sooner: then it lands on its words
  const rowAt = landings(words, rows, len, rows.length ? Math.min(10, saidFrom(words, rows[0])) : 10, true);
  let strike = len - 14;
  if (strikeSay) try { strike = findPhrase(words, strikeSay); } catch (e) { throw new Error(`<Strike> strikeSay: ${(e as Error).message}`); }
  // what's struck is read first: the strike waits until the last row has been up half a second (a strikeSay on the
  // card's own words struck it as it landed), while the part is still up
  if (rows.length) strike = Math.min(Math.max(strike, rowAt[rows.length - 1] + 15), len - 8);
  const card = rowAt[0];
  const rowSize = Math.min(74, ...rows.map((r) => fit(r, 820, 74)));
  const h = rows.length > 1 ? key(f, { [rowAt[1] - 1]: 120, [rowAt[1] + 5]: 192 }) : 120;
  const top = TOP + ls.length * 140 + 10;
  return (
    <>
      <div style={{ position: "absolute", left: 90, top: TOP, width: 900, display: "flex", flexDirection: "column" }}>
        {ls.map((l, i) => <LockLine key={i} line={l} at={0} f={f} size={fit(l, 900, i === 0 ? 130 : 110)} />)}
      </div>
      {f >= card ? (
        <div style={{ position: "absolute", left: 100, top, width: 880, height: h, boxSizing: "border-box", padding: "22px 20px 0", borderRadius: 32, backgroundColor: CARD, overflow: "hidden", transform: `translateY(${key(f - card, { 0: 30, 6: 0 })}px)` }}>
          {rows.map((r, i) => (
            <div key={i} style={{ position: "relative", height: 72, opacity: f >= rowAt[i] ? 1 : 0 }}>
              <div style={{ ...type(rowSize, "#e2e2e2", -1, false), textAlign: "center", transform: `translateY(${rise(f - rowAt[i])}px)` }}>{r}</div>
              <div style={{ position: "absolute", left: "50%", top: rowSize * 0.42, height: 9, borderRadius: 5, backgroundColor: STRIKE, width: key(f - strike - 4 * i, { 0: 0, 8: Math.min(820, ems(r) * rowSize) }), transform: "translateX(-50%)" }} />
            </div>
          ))}
        </div>
      ) : null}
      <Hit at={card} sound={sound} fallback="soft paper swipe whoosh, short" gain={-22} screen={BLIP} />
      <Hit at={strike} sound={sound} fallback="marker pen scribble strike-through on paper, quick" gain={-20} seconds={0.8} />
    </>
  );
});

export interface PhrasesProps extends Cue {
  /** the lines, " / " between them, each a short phrase the speaker says in turn; *accent* words lit */
  text: string;
}

/** Big lines on purpose: a run of short phrases in the top band, one at a time, each landing on its words and held
 *  until the next, each as big as it fits (the reference's thesis, said phrase by phrase). */
export const Phrases = part<PhrasesProps>("Phrases", PARTS.Phrases, ({ text, sound, span }) => {
  const f = useCurrentFrame();
  const TOP = useContext(TopContext);
  const ls = lines(text);
  const at = landings(useWords(), ls, span.to - span.from, 0, true, "Phrases", span.from);
  const i = at.reduce((n, a, k) => (f >= a ? k : n), 0);
  const local = f - at[i];
  return (
    <>
      <div key={i} style={{ position: "absolute", left: 60, top: TOP, width: W, textAlign: "center", ...type(fit(ls[i], W, 150), OVER), opacity: local >= 0 ? 1 : 0, transform: `translateY(${key(local, { 0: 26, 5: 0 })}px) scale(${key(local, { 0: 0.8, 3: 1.06, 6: 1 })})` }}>
        {accent(ls[i])}
      </div>
      <Hit sound={sound} fallback="deep punchy cinematic impact hit, tight, no reverb tail" gain={-18} seconds={1.2} screen={THUD} />
    </>
  );
});

export interface CalloutProps extends Cue {
  /** the small line above: "Step 1", "Fact" */
  label?: string;
  title: string;
  detail?: string;
}

/** A labelled card in the top band for a step or a claim. */
export const Callout = part<CalloutProps>("Callout", PARTS.Callout, ({ label, title, detail, sound }) => {
  const f = useCurrentFrame();
  const TOP = useContext(TopContext);
  return (
    <>
      <div style={{ position: "absolute", left: 60, top: TOP, width: W, boxSizing: "border-box", padding: "34px 50px", borderRadius: 44, backgroundColor: CARD, boxShadow: "0 20px 60px rgba(0,0,0,0.35)", transform: `translateY(${key(f, { 0: -30, 7: 0 })}px)`, opacity: key(f, { 0: 0, 4: 1 }) }}>
        {label ? <div style={{ fontFamily: BODY, fontWeight: 800, fontSize: 38, letterSpacing: 2, color: ACCENT, textTransform: "uppercase", marginBottom: 12 }}>{label}</div> : null}
        <div style={{ ...type(fit(title, 860, 96), INK, -1, false), whiteSpace: "normal" }}>{accent(title)}</div>
        {detail ? <div style={{ fontFamily: BODY, fontWeight: 600, fontSize: 40, lineHeight: 1.25, color: "#d8d8d8", marginTop: 14 }}>{accent(detail)}</div> : null}
      </div>
      <Hit sound={sound} fallback="fast clean whoosh swipe transition, short, airy" gain={-21} screen={BLIP} />
    </>
  );
});

export interface FollowProps extends Cue {
  text: string;
}

/** The ask as a pill in the top band: "+ follow for more". */
export const Follow = part<FollowProps>("Follow", PARTS.Follow, ({ text, sound }) => {
  const f = useCurrentFrame();
  const TOP = useContext(TopContext);
  // the pill is the accent already and holds one line: stars and " / " read as plain words
  const said = plain(text).split(/\s*\/\s*/).join(" ");
  return (
    <>
      <div style={{ position: "absolute", left: 108, top: TOP + 64, width: 813, padding: 26, borderRadius: 60, backgroundColor: ACCENT, color: ON_ACCENT, fontFamily: BODY, fontWeight: 800, fontSize: Math.min(64, Math.floor(800 / (0.62 * said.length))), lineHeight: 1, letterSpacing: 2, textTransform: "uppercase", textAlign: "center", whiteSpace: "nowrap", boxShadow: "0 10px 30px rgba(0,0,0,0.35)", transform: `scale(${key(f, POP)})` }}>
        {said}
      </div>
      <Hit sound={sound} fallback="bubbly UI pop click, satisfying, short" gain={-17} seconds={0.6} screen={[BLIP[0], -3, 0.6]} />
    </>
  );
});

export interface EndCardProps extends Cue {
  name: string;
  /** a logo image (optional) */
  logo?: string;
  line?: string;
  /** the ask: "Shop now", "Link in bio" */
  cta?: string;
}

/** The closing card over the whole frame: the name or logo, one line, and the ask. */
export const EndCard = part<EndCardProps>("EndCard", PARTS.EndCard, ({ name, logo, line, cta, sound }) => {
  const t = key(useCurrentFrame(), { 0: 0, 10: 1 });
  return (
    <>
      <AbsoluteFill style={{ opacity: t, backgroundColor: "rgba(10,10,10,0.92)", display: "flex", flexDirection: "column", alignItems: "center", justifyContent: "center", gap: 40, padding: 80 }}>
        {logo ? <Img src={logo} style={{ height: 180, objectFit: "contain" }} /> : <div style={{ ...type(fit(name, 900, 180), INK), textShadow: SHADOW }}>{name}</div>}
        {line ? <div style={{ ...type(fit(line, 900, 80), INK, -1, false), whiteSpace: "normal", textAlign: "center" }}>{accent(line)}</div> : null}
        {cta ? <div style={{ padding: "26px 60px", borderRadius: 60, backgroundColor: ACCENT, color: ON_ACCENT, fontFamily: BODY, fontWeight: 800, fontSize: 56, letterSpacing: 2, textTransform: "uppercase", transform: `scale(${t})` }}>{cta}</div> : null}
      </AbsoluteFill>
      <Hit sound={sound} fallback="soft rising whoosh into a gentle chime, short" gain={-20} seconds={1.2} screen={THUD} />
    </>
  );
});
