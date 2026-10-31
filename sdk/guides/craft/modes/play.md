# Mode: play

Games and playable scenes: pieces whose job is that someone wants another go.

## It works when

A new player understands what to do within two seconds of looking, their first input does something they can see
and hear, and when it ends they press to play again.

## What to make

- **It plays live.** A game always runs live on the stage, never as a video in its place. If it's too slow, fix the
  cost, not the liveness.
- **Readable in two seconds.** The player, the goal and the danger read at a glance: by shape and contrast first,
  colour second. One look should tell you what you are.
- **The first input answers.** The first key or touch moves something, with a sound. A screen that ignores the
  player loses them.
- **Feel before features.** Get the core move right (its speed, weight, sound, and the frame it lands on) before
  adding a second one.
- **Clear failure, quick restart.** The player sees why they lost and is playing again in a moment.
- **Deterministic.** State lives in `useGameState` with a pure step, so a logged play renders back to the same pixels.

## The rut

A title screen before anything happens. Controls explained in text instead of taught by the first obstacle. A score
that means nothing.

## Check

- Playtest it live, with real input, not from a rendered video.
- Harden the edges: two keys at once, holding a key, losing focus, the very first frame and the last.
