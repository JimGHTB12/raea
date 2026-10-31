// P15: Web Audio's decodeAudioData, as a waveform reads a file's sound: WAV files made here (a stereo one at the
// context's 44100 Hz, decoded sample for sample, and a mono one at 22050 Hz, resampled to twice its length), decoded by
// an OfflineAudioContext into AudioBuffers whose channels, rates, lengths, durations, first samples and peaks are drawn.
// A buffer that isn't a sound is refused (EncodingError), and a buffer made by hand reads back what was copied in.
// proves: api.OfflineAudioContext, api.OfflineAudioContext.OfflineAudioContext, api.BaseAudioContext.decodeAudioData, api.BaseAudioContext.decodeAudioData.returns_promise, api.AudioBuffer, api.AudioBuffer.AudioBuffer, api.AudioBuffer.getChannelData, api.AudioBuffer.numberOfChannels, api.AudioBuffer.sampleRate, api.AudioBuffer.length, api.AudioBuffer.duration, api.AudioBuffer.copyToChannel, api.AudioBuffer.copyFromChannel
import { useEffect, useState } from "react";
import { trace } from "parity";
export const steps = [{ wait: 30 }];

/** A 16-bit PCM WAV of `channels`, each a function of the sample's index, `n` samples at `rate`. */
function wav(rate: number, n: number, channels: ((i: number) => number)[]): ArrayBuffer {
  const c = channels.length;
  const b = new DataView(new ArrayBuffer(44 + n * c * 2));
  const str = (at: number, s: string) => [...s].forEach((ch, i) => b.setUint8(at + i, ch.charCodeAt(0)));
  str(0, "RIFF");
  b.setUint32(4, 36 + n * c * 2, true);
  str(8, "WAVEfmt ");
  b.setUint32(16, 16, true);
  b.setUint16(20, 1, true);
  b.setUint16(22, c, true);
  b.setUint32(24, rate, true);
  b.setUint32(28, rate * c * 2, true);
  b.setUint16(32, c * 2, true);
  b.setUint16(34, 16, true);
  str(36, "data");
  b.setUint32(40, n * c * 2, true);
  for (let i = 0; i < n; i++) channels.forEach((f, k) => b.setInt16(44 + (i * c + k) * 2, Math.round(f(i) * 32767), true));
  return b.buffer;
}

const sine = (rate: number, hz: number, amp: number) => (i: number) => amp * Math.sin((2 * Math.PI * hz * i) / rate);
const peak = (d: Float32Array) => d.reduce((m, v) => Math.max(m, Math.abs(v)), 0);
const describe = (b: AudioBuffer) =>
  `${b.numberOfChannels} ch, ${b.sampleRate} Hz, ${b.length} frames, ${b.duration.toFixed(3)} s; ` +
  Array.from({ length: b.numberOfChannels }, (_, c) => `ch${c} peak ${peak(b.getChannelData(c)).toFixed(2)} first ${Array.from(b.getChannelData(c).slice(0, 3), (v) => v.toFixed(3)).join(" ")}`).join("; ");

export default function Case() {
  const [lines, setLines] = useState<string[]>([]);
  useEffect(() => {
    const ctx = new OfflineAudioContext(1, 1, 44100);
    const said: string[] = [];
    const say = (l: string) => {
      said.push(l);
      trace("line", "decode", l);
      setLines([...said]);
    };
    (async () => {
      const stereo = await ctx.decodeAudioData(wav(44100, 4410, [sine(44100, 441, 0.5), sine(44100, 882, 0.25)]));
      say(`stereo: ${describe(stereo)}`);
      const mono = await ctx.decodeAudioData(wav(22050, 2205, [sine(22050, 100, 0.8)]));
      say(`mono at 22050: ${mono.numberOfChannels} ch, ${mono.sampleRate} Hz, ${mono.length} frames, peak ${peak(mono.getChannelData(0)).toFixed(1)}`);
      await ctx.decodeAudioData(new Uint8Array([1, 2, 3, 4, 5, 6, 7, 8]).buffer).then(
        () => say("noise: decoded"),
        (e) => say(`noise: ${(e as Error).name}`),
      );
      const made = new AudioBuffer({ numberOfChannels: 2, length: 4, sampleRate: 8000 });
      made.copyToChannel(new Float32Array([0.5, -0.5, 0.25, 0]), 1);
      const back = new Float32Array(4);
      made.copyFromChannel(back, 1);
      say(`made: ${made.numberOfChannels} ch, ${made.length} frames, ${made.duration} s, ch1 ${Array.from(back).join(" ")}, ch0 ${Array.from(made.getChannelData(0)).join(" ")}`);
    })();
  }, []);
  return (
    <div data-testid="root" className="h-full w-full bg-white p-4 text-xs">
      <ul data-testid="lines">
        {lines.map((l, i) => (
          <li key={i} className="mb-1">
            {l}
          </li>
        ))}
      </ul>
    </div>
  );
}
