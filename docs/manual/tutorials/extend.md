---
title: "Extend raea with the SDK"
summary: "Compositions and controllers are TypeScript on raea's SDK: write one, render it, post it to the feed with the feed's tools, or ask your agent to make new feed items and controllers for you."
order: 8
sources: ["sdk/raea/src", "sdk/raea/src/brief.ts", "apps/app/src/acp/protocol.ts", "docs/manual/examples/title.tsx", "apps/workspace/src/controllers.rs"]
tutorial: "extend"
---

Everything raea makes and every setting it keeps is TypeScript you can read and write. This tutorial writes a small
composition, renders it, posts it to the feed, writes a controller, and then hands the same jobs to an agent.

## The SDK

raea's SDK is TypeScript. A composition imports from raea, and its sub-paths hold the rest: cameras, lights, materials,
physics, text and sound. Because it's ordinary TypeScript, your agent writes it the way it writes any other code, and
you can read every line it wrote.

## A composition

Here is a small composition, a title. It registers one composition with its size, frame rate and length, and each word
springs up into place four frames after the one before it. raea renders it frame by frame. Render one frame as a
still to look at it, then render the whole piece to an MP4.

## Post it to the feed

The agents raea starts get the feed's own tools. Feed post puts a render, a still, a sound or a game into the project's
feed. It takes a title and the file, and to post a new version, it names the item it replaces, with one line on what
changed. Feed propose suggests something to make next, for you to accept first.

## A controller from code

A controller is a TypeScript file too. It exports controller, from raea slash brief, with groups of typed fields. Write
one, then run raea controller write, and it's on the Controllers page for every agent to read. Agents in the app use
the controller write tool, and the page shows the new controller as soon as they do.

## Ask your agent instead

So you rarely write any of this yourself. Ask your agent in plain words, in New chat. It writes the composition, checks
it, renders it, and posts it to the feed. Ask for a controller, and it builds one and puts it on the page. You review
what lands, and leave notes, as you would on anything else.

## Extend the engine

Underneath, raea runs on the Bevy game engine. The SDK's sub-paths follow Bevy's own crates, so a Bevy example's ideas
map straight onto raea, and the SDK reference lists every import path and what it exports. When you need something the
SDK doesn't have yet, write it as your own component, and every composition can import it.

```tsx
// docs/manual/examples/title.tsx, in short: each word springs up four frames after the last
const p = spring({ frame, fps, delay: i * 4, config: { damping: 14, stiffness: 160 } });
registerRoot(() => <Composition id="title" component={Title} width={1920} height={1080} fps={30} durationInFrames={60} />);
```

```sh
raea render title.tsx --frame 30 -o title.png     # one frame, to look at
raea render title.tsx -o title.mp4                # the whole piece
raea controller write controllers/review.ts       # a controller, for every agent
```

```ts
// controllers/review.ts
import { controller } from "raea/brief";

export default controller({
  title: "Review",
  groups: [
    {
      title: "Sign-off",
      fields: [
        { label: "Watched to the end by a person", type: "toggle", value: true },
        { label: "A redo is", type: "choice", options: ["a new version", "a new item"], value: "a new version" },
      ],
    },
  ],
});
```

Next: [Work as a team](teams.md).
