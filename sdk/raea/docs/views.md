# Pictures of an edit (raea/views)

Components for showing footage being worked on: an editor's timeline, a graph of embeddings in 3D, and perception
overlays. They draw no text. Each one is a pure function of its props, drawn in the page's DOM and SVG, so a frame
renders the same every time and needs no 3D pass. Animate them by passing props computed from the frame or the song
(`useSong`). The film's editing sequence (`content/anything/edit/scene.tsx`) uses all of them with raea's real
perception output.

```tsx
import { CutStrip, EmbeddingGraph, FaceBox, HandSkeleton, Timeline, Waveform, WordTrack, projector, snapTo } from "raea/views";
```

## Timeline

An editor's timeline with no words on it: a ruler of beat ticks, tracks of clips, notches, a playhead and a blade.

```tsx
<Timeline width={1920} height={560} view={[t0, t1]}            // the seconds shown: zoom and scroll by moving them
  tracks={[
    { height: 170, clips: [{ from: 1.2, to: 3.0, color: "#2343ff", thumbs: stills, marks: wordBounds }] },
    { height: 90, clips: [{ from: 0, to: 14, color: "#f3ead8", wave: peaks }] },
  ]}
  playhead={songTime} beats={map.beats} downbeats={map.downbeats}
  flash={(i) => (t >= map.beats[i] ? Math.max(0, 1 - (t - map.beats[i]) / 0.2) : 0)}
  blade={cutAt} theme={{ playhead: "#ffd400", blade: "#ff3b1f" }} />
```

- A clip's `thumbs` are drawn along it as a film strip; `wave` (peaks 0..1) as bars; `marks` (timeline seconds) as
  notches, such as its word boundaries before a trim.
- `lift` and `shift` move a clip off its track while it's dragged; `flip` turns it upside down; `selected` (0..1)
  outlines it.
- Times are the timeline's, so put the song's beats on it directly and snap clip edges with `snapTo(beats, t)`.

## EmbeddingGraph

Nodes (stills or dots) at their embedding's 3D places, seen through an orbit camera, with edges between them.

```tsx
<EmbeddingGraph width={1920} height={1080} fog={5}
  camera={{ target: [0, 0, 0], yaw: f / 90, pitch: 0.2, distance: 3.3, fov: 50 }}
  nodes={clips.map((c) => ({ pos: c.xyz, src: c.still, color: clusterColour[c.k], aspect: c.aspect, lit: hits.has(c.id) ? 1 : 0 }))}
  edges={[...neighbours, ...queryPath.map(([a, b], r) => ({ a, b, drawn: clamp(progress * 14 - r), lit: 1 }))]} />
```

- Lay the vectors out yourself (UMAP or PCA to 3D, then normalise each axis so the cloud fills the view);
  `content/anything/edit/tools/graph.py` does it for SigLIP vectors from `raea-meaning ingest`.
- `drawn` (0..1) draws an edge in from `a` toward `b`; `lit` (0..1) grows a node and makes it glow in `litColor`;
  `opacity` 0 hides it.
- `projector(camera, w, h)` and `cameraEye(camera)` give the same projection, for drawing your own marks over the
  graph (a pulse at a node).

## Perception overlays

| component | what it draws | from |
|---|---|---|
| `FaceBox` | corner brackets locking onto a box (`lock` 0..1, flying in from 1.6x with an overshoot) | `raea-perceive faces` |
| `HandSkeleton` | a hand's 21 points and bones (`drawn` 0..1 grows them from the wrist) | `raea-perceive hands` |
| `CutStrip` | a film strip scrolling, each cut's line snapping across it, shot-end bars under it | `raea-perceive transnet` |
| `WordTrack` | words as blocks as long as they're said; the one being said lit | Parakeet's words (`<stem>.words.json`) |
| `Waveform` | mirrored bars, the played part in another colour | a clip's peaks |

Boxes and points are in pixels of the box you place the overlay in: multiply perception's 0..1 boxes by its size.

## Gotchas

- Everything is DOM, so hundreds of nodes are fine but each still is an image load: give `src` small stills (200 px).
- The graph draws far to near with no depth test between a node and the edges in front of it: edges are always
  under the nodes.
- None of these draw type. If a beat needs a word, it belongs to the footage (a transcript), not a label.
