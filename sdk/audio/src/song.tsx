// A song's energy map, frame by frame (spec: "Pacing to a song's energy"). The map is the JSON `raea song
// FILE -o song.json` writes (systems/song/src/song/): beats, downbeats and bars from Beat This!, per-beat features
// 0..1 (loudness, onsets, brightness, low end, high band, energy), sections (the file's own [Section] markers when it
// has them, else novelty) and events (build, drop, stop, hit, dip, break), each section and bar with a suggested cut
// rate. Everything here is a pure function of the map and the frame.
//
//   import map from "./song.json";
//   const s = useSong(map);                     // frame 0 of the enclosing Sequence is the song's start
//   s.energy, s.beatInBar, s.section?.name, s.inBuild && s.buildProgress, s.untilDrop
//   const cuts = song(map, { fps }).cutsAt();   // frames to cut on, paced to the energy
export { songAt, song, useSong, SongPlot } from "./song/all";
export type { SongEventKind, SongLevel, SongHit, SongEvent, SongSection, SongBar, SongFeatures, SongMap, SongMoment, SongOptions, FrameRange, CutOptions, Song, UseSongOptions, SongPlotProps } from "./song/all";


