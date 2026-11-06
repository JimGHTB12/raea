# This project

A video, laid out by feature as a Bevy project is (raea init --layout feature):

- space.json: its name and kind. package.json: raea.plugins, the Bevy, Blitz and raea plugins it uses.
- assets/: its files. `staticFile("x.png")` reads assets/x.png (a media/ folder beside it is read first).
- src/main.tsx: the App, rendered with `raea render src/main.tsx`. It registers the Composition.
- src/<feature>.tsx: one feature each (title, captions, music), its components together; one with work in the App
  each frame also exports its plugin (definePlugin from "raea/app").
- src/shots/: the shots, each a scene component placed in a Sequence. src/screens/: one file per top-level state.
  src/ui/: page parts shared between features.

Never a components/ folder: a feature file holds a feature's components by name.
