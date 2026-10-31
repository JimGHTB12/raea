// raea/tone: Tone.js's API, running on raea's one mixer core (no Web Audio). `import * as Tone from
// "raea/tone"` and write Tone as its docs show: instruments, effects, channels, connect/toDestination/chain,
// triggerAttackRelease, the Transport with Loop, Part, Sequence and Pattern, Tone's time notation.
//
// What's different, because a composition is a function of its frame (sdk/react/docs/audio-stack.md, "Tone.js"):
// - Nothing is scheduled on a wall clock. Tone code builds a graph and an event list; the Transport's callbacks run in
//   time order as the frames need them; each event lands on its own sample of the timeline. A render and a play of
//   the same edit are the same samples.
// - Time 0 is the composition's frame 0. Tone.now() is 0 at the top level and the event's time inside a callback.
// - The graph (instruments, effects, channels and how they connect) is the one the edit has when its first frame
//   renders: make instruments and effects at the top level. Param changes and ramps after that are events.
// - Draw.schedule, Offline, analysers and user media have no meaning here (see the compatibility table).
// raea's additions: buses (music, sfx, voice, ambience, ui) with ducking, generated sources (sao, tokensynth) for
// Sampler and Player, seeded variation, <Sfx> for a game's one-shots, and Time(...).toFrames().
import { fish, lyria, sao } from "@raea/asset/assets";
export { getContext, context, setContext, start, loaded, now, immediate, version, mtof, ftom, dbToGain, gainToDb, intervalToFrequencyRatio, Frequency, Midi, Time, TransportTime, Ticks, Param, Signal, ToneAudioNode, getDestination, Destination, Master, Instrument, Synth, MonoSynth, FMSynth, AMSynth, DuoSynth, MembraneSynth, NoiseSynth, MetalSynth, PluckSynth, PolySynth, Sampler, Player, GrainPlayer, Players, Oscillator, OmniOscillator, FatOscillator, PulseOscillator, Noise, Effect, Reverb, Freeverb, JCReverb, FeedbackDelay, PingPongDelay, Filter, AutoFilter, Compressor, Limiter, EQ3, Distortion, Chorus, BitCrusher, Tremolo, Vibrato, Phaser, AutoPanner, Gain, Volume, Panner, Channel, PanVol, Solo, Meter, FFT, Waveform, Analyser, LFO, Convolver, PitchShift, Offline, bus, buses, tokensynth, getTransport, Transport, Draw, Loop, Part, Sequence, Pattern, ToneOut, Sfx, vary, snapshot, reset } from "./tone/all";

export { sao, lyria, fish };
