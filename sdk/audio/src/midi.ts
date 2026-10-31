// Standard MIDI Files in and out (moved from music.tsx so the feed's host can read and write them without the Tone
// runtime): pure, no imports. @raea/audio/music re-exports all of it and writes its Parts with writeMidi (toMidi).

// In @tonejs/midi's JSON shape (tracks[].notes[] with midi, time, duration and
// velocity in seconds and 0-1; header.tempos, timeSignatures, ppq), with no dependency.

export type MidiNote = { midi: number; time: number; duration: number; velocity: number; ticks: number; durationTicks: number };
export type MidiTrack = { name: string; channel: number; notes: MidiNote[] };
export type MidiFile = { header: { ppq: number; tempos: { ticks: number; bpm: number }[]; timeSignatures: { ticks: number; timeSignature: [number, number] }[]; name: string }; tracks: MidiTrack[]; duration: number };

/** A Standard MIDI File read (format 0 or 1): notes in seconds through its tempo map. */
export function parseMidi(bytes: Uint8Array): MidiFile {
  let p = 0;
  const u32 = () => ((bytes[p++] << 24) | (bytes[p++] << 16) | (bytes[p++] << 8) | bytes[p++]) >>> 0;
  const u16 = () => (bytes[p++] << 8) | bytes[p++];
  const vlq = () => {
    let v = 0;
    for (;;) {
      const b = bytes[p++];
      v = (v << 7) | (b & 127);
      if (!(b & 128)) return v;
    }
  };
  const tag = () => String.fromCharCode(bytes[p], bytes[p + 1], bytes[p + 2], bytes[p + 3]);
  if (tag() !== "MThd") throw new Error("parseMidi: not a MIDI file");
  p += 4;
  const hlen = u32();
  const hstart = p;
  u16();
  const ntracks = u16();
  const ppq = u16();
  p = hstart + hlen;
  const tempos: { ticks: number; bpm: number }[] = [];
  const sigs: { ticks: number; timeSignature: [number, number] }[] = [];
  const raw: { name: string; channel: number; notes: { midi: number; ticks: number; durationTicks: number; velocity: number }[] }[] = [];
  let name = "";
  for (let k = 0; k < ntracks && p < bytes.length; k++) {
    if (tag() !== "MTrk") throw new Error("parseMidi: a track without MTrk");
    p += 4;
    const len = u32();
    const end = p + len;
    let ticks = 0;
    let status = 0;
    const open = new Map<number, { ticks: number; velocity: number }[]>();
    const track = { name: "", channel: 0, notes: [] as { midi: number; ticks: number; durationTicks: number; velocity: number }[] };
    while (p < end) {
      ticks += vlq();
      let b = bytes[p];
      if (b & 128) {
        status = b;
        p++;
      }
      b = status;
      if (b === 0xff) {
        const type = bytes[p++];
        const l = vlq();
        const data = bytes.subarray(p, p + l);
        p += l;
        if (type === 0x51) tempos.push({ ticks, bpm: 60_000_000 / ((data[0] << 16) | (data[1] << 8) | data[2]) });
        else if (type === 0x58) sigs.push({ ticks, timeSignature: [data[0], 2 ** data[1]] });
        else if (type === 0x03) {
          track.name = String.fromCharCode(...data);
          if (k === 0) name = track.name;
        }
      } else if (b === 0xf0 || b === 0xf7) {
        p += vlq();
      } else {
        const kind = b & 0xf0;
        const ch = b & 15;
        const d1 = bytes[p++];
        const d2 = kind === 0xc0 || kind === 0xd0 ? 0 : bytes[p++];
        if (kind === 0x90 && d2 > 0) {
          track.channel = ch;
          (open.get(d1) ?? open.set(d1, []).get(d1)!).push({ ticks, velocity: d2 / 127 });
        } else if (kind === 0x80 || (kind === 0x90 && d2 === 0)) {
          const on = open.get(d1)?.shift();
          if (on) track.notes.push({ midi: d1, ticks: on.ticks, durationTicks: ticks - on.ticks, velocity: on.velocity });
        }
      }
    }
    p = end;
    raw.push(track);
  }
  if (!tempos.length) tempos.push({ ticks: 0, bpm: 120 });
  tempos.sort((a, b) => a.ticks - b.ticks);
  // ticks to seconds through the tempo map
  const secs = (t: number) => {
    let s = 0;
    for (let i = 0; i < tempos.length; i++) {
      const from = tempos[i].ticks;
      const to = i + 1 < tempos.length ? Math.min(tempos[i + 1].ticks, t) : t;
      if (to <= from) break;
      s += ((to - from) / ppq) * (60 / tempos[i].bpm);
    }
    return s;
  };
  const tracks = raw.map((tr) => ({
    name: tr.name,
    channel: tr.channel,
    notes: tr.notes.sort((a, b) => a.ticks - b.ticks || a.midi - b.midi).map((n) => ({ midi: n.midi, ticks: n.ticks, durationTicks: n.durationTicks, velocity: n.velocity, time: secs(n.ticks), duration: secs(n.ticks + n.durationTicks) - secs(n.ticks) })),
  }));
  const duration = Math.max(0, ...tracks.flatMap((t) => t.notes.map((n) => n.time + n.duration)));
  return { header: { ppq, tempos, timeSignatures: sigs, name }, tracks, duration };
}

/** a track to write: its name, channel (9 for drums) and notes in seconds */
export type MidiTrackOut = { name: string; channel: number; notes: { midi: number; time: number; duration: number; velocity: number }[] };

/** Tracks written as a Standard MIDI File (format 1, the tempo track first) at one tempo and time signature. */
export function writeMidi(clock: { bpm: number; timeSignature: [number, number]; ppq: number }, tracks: MidiTrackOut[]): Uint8Array {
  const out: number[] = [];
  const u32 = (v: number) => out.push((v >>> 24) & 255, (v >>> 16) & 255, (v >>> 8) & 255, v & 255);
  const vlq = (v: number, into: number[]) => {
    const b = [v & 127];
    while ((v >>= 7)) b.unshift((v & 127) | 128);
    into.push(...b);
  };
  const toTicks = (seconds: number) => Math.round((seconds / (60 / clock.bpm)) * clock.ppq);
  const chunks: number[][] = [];
  // the tempo track
  const tempo = Math.round(60_000_000 / clock.bpm);
  chunks.push([0, 0xff, 0x51, 3, (tempo >> 16) & 255, (tempo >> 8) & 255, tempo & 255, 0, 0xff, 0x58, 4, clock.timeSignature[0], Math.round(Math.log2(clock.timeSignature[1])), 24, 8, 0, 0xff, 0x2f, 0]);
  for (const t of tracks) {
    const evs: [number, number, number, number][] = [];
    for (const n of t.notes) {
      const vel = Math.max(1, Math.min(127, Math.round(n.velocity * 127)));
      evs.push([toTicks(n.time), 1, n.midi, vel], [toTicks(n.time + n.duration), 0, n.midi, 0]);
    }
    // offs before ons at the same tick
    evs.sort((a, b) => a[0] - b[0] || a[1] - b[1]);
    const tr: number[] = [];
    const nm = [...t.name].map((c) => c.charCodeAt(0) & 127);
    tr.push(0, 0xff, 0x03, nm.length, ...nm);
    let last = 0;
    for (const [tick, on, note, vel] of evs) {
      vlq(tick - last, tr);
      last = tick;
      tr.push((on ? 0x90 : 0x80) | t.channel, note, vel);
    }
    tr.push(0, 0xff, 0x2f, 0);
    chunks.push(tr);
  }
  out.push(0x4d, 0x54, 0x68, 0x64);
  u32(6);
  out.push(0, 1, (chunks.length >> 8) & 255, chunks.length & 255, (clock.ppq >> 8) & 255, clock.ppq & 255);
  for (const tr of chunks) {
    out.push(0x4d, 0x54, 0x72, 0x6b);
    u32(tr.length);
    out.push(...tr);
  }
  return new Uint8Array(out);
}
