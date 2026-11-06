# animate

Notes like these: "it's static", "bring it to life", "make it move", "it feels like a slideshow". Motion added to
something that holds still, so each move says something.

## Assess
- Find what the motion should say at each moment: arrival, emphasis, cause and effect, a change of state. A part
  with nothing to say stays still.
- Find the beats it lands on: the voice's words, the music's hits (`raea song FILE --plot`), the cuts.

## Change
- Animate entrances, the one emphasis per beat and exits; leave everything else still.
- One easing family for the whole piece; springs for things with weight, short eases for type (`raea guide
  motion/animating`).
- Stagger related parts by a few frames instead of moving them at once.
- Everything reads the frame (`useCurrentFrame()`), every `interpolate` clamped.

## Verify
- Play it at full speed, then step the frames around each beat: each move starts on its cue and settles before
  the next.
- Nothing moves without a reason you could say in a few words.
