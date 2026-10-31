//! Pocket TTS (Kyutai, 100M, CC-BY-4.0) on ONNX, streamed: each few 80 ms Mimi frames go out as soon as they're decoded.
//!
//! The export is KevinAHM/pocket-tts-onnx's english_2026-04 bundle (int8 flow LM, flow head and Mimi decoder; the text
//! conditioner in fp32), and this is its pocket_tts_onnx.py loop in Rust: the voice's flow-LM state (a predefined voice
//! is a saved KV cache, kyutai/pocket-tts-without-voice-cloning) restored once per voice; per sentence, the text
//! embeddings through the flow LM, then one latent per frame (the flow head's LSD step from Gaussian noise at
//! temperature 0.7) until the EOS logit passes -4 and a few frames more; the Mimi decoder turns each two latents into
//! 3840 samples at 24 kHz, its own streaming state carried across the sentence. The tokenizer is mlx-community/pocket-tts's
//! tokenizer.json, which encodes exactly as the bundle's SentencePiece model.
//!
//! A voice can also be a clip: any WAV in ~/.raea-feed/voices is a voice named after its file. Its audio goes through
//! the bundle's Mimi encoder (speaker projection included) behind the BOS embedding, and the flow LM's state after that
//! prompt is the voice, computed once per voice (pocket_tts get_state_for_audio_prompt; its 30 s cap, plus a trim of
//! the silence at either end).

use std::borrow::Cow;
use std::collections::HashMap;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};

use anyhow::{anyhow, bail, Context};
use ort::session::{Session, SessionInputValue};
use ort::value::{DynValue, Tensor};
use serde_json::Value;

const BUNDLE: &str = "KevinAHM/pocket-tts-onnx";
const VOICE_REPO: &str = "kyutai/pocket-tts-without-voice-cloning";

/// Frames (80 ms each) decoded per chunk: small, so the first sound comes after 160 ms of audio is generated.
const CHUNK_FRAMES: usize = 2;
const TEMPERATURE: f32 = 0.7;
/// The flow head's decode steps (the export's default).
const LSD_STEPS: usize = 1;
const EOS_LOGIT: f32 = -4.0;
/// The reference's cap on an audio prompt (get_state_for_audio_prompt's truncate).
const MAX_PROMPT_SECONDS: usize = 30;

/// One state tensor's initial value, kept in Rust so each sentence can start from a fresh copy.
#[derive(Clone)]
pub(crate) enum Data {
    F32(Vec<f32>),
    I64(Vec<i64>),
    Bool(Vec<bool>),
}

#[derive(Clone)]
pub(crate) struct Init {
    name: String,
    shape: Vec<i64>,
    data: Data,
}

impl Init {
    /// A state output kept as a fresh starting value.
    fn from_value(name: String, v: &DynValue) -> anyhow::Result<Init> {
        if let Ok((shape, d)) = v.try_extract_tensor::<f32>() {
            return Ok(Init { name, shape: shape.iter().copied().collect(), data: Data::F32(d.to_vec()) });
        }
        if let Ok((shape, d)) = v.try_extract_tensor::<i64>() {
            return Ok(Init { name, shape: shape.iter().copied().collect(), data: Data::I64(d.to_vec()) });
        }
        let (shape, d) = v.try_extract_tensor::<bool>()?;
        Ok(Init { name, shape: shape.iter().copied().collect(), data: Data::Bool(d.to_vec()) })
    }

    fn value(&self) -> anyhow::Result<DynValue> {
        let shape = self.shape.clone();
        Ok(match &self.data {
            Data::F32(v) => Tensor::from_array((shape, v.clone()))?.into_dyn(),
            Data::I64(v) => Tensor::from_array((shape, v.clone()))?.into_dyn(),
            Data::Bool(v) => Tensor::from_array((shape, v.clone()))?.into_dyn(),
        })
    }
}

/// A bundle.json state manifest as fresh tensors (its fills: nan, zeros, ones, empty).
fn manifest(entries: &Value) -> anyhow::Result<Vec<(Init, String, String)>> {
    let mut out = Vec::new();
    for e in entries.as_array().context("manifest")? {
        let shape: Vec<i64> = e["shape"].as_array().context("shape")?.iter().filter_map(Value::as_i64).collect();
        let n: usize = shape.iter().map(|&d| d as usize).product();
        let fill = e["fill"].as_str().unwrap_or("zeros");
        let data = match e["dtype"].as_str().unwrap_or("") {
            "float32" => Data::F32(vec![
                match fill {
                    "nan" => f32::NAN,
                    "ones" => 1.0,
                    _ => 0.0,
                };
                n
            ]),
            "int64" => Data::I64(vec![if fill == "ones" { 1 } else { 0 }; n]),
            "bool" => Data::Bool(vec![fill == "ones"; n]),
            other => bail!("state dtype {other}"),
        };
        let name = e["input_name"].as_str().context("input_name")?.to_string();
        let module = e["module"].as_str().unwrap_or("").to_string();
        let key = e["key"].as_str().unwrap_or("").to_string();
        out.push((Init { name, shape, data }, module, key));
    }
    Ok(out)
}

/// A safetensors file's tensors (F32 and I64 only, all a voice state holds): name -> (shape, data).
fn safetensors(file: &Path) -> anyhow::Result<HashMap<String, (Vec<i64>, Data)>> {
    let bytes = std::fs::read(file).with_context(|| file.display().to_string())?;
    let n = u64::from_le_bytes(bytes[..8].try_into()?) as usize;
    let header: Value = serde_json::from_slice(&bytes[8..8 + n])?;
    let body = &bytes[8 + n..];
    let mut out = HashMap::new();
    for (name, t) in header.as_object().context("safetensors header")? {
        if name == "__metadata__" {
            continue;
        }
        let shape: Vec<i64> = t["shape"].as_array().context("shape")?.iter().filter_map(Value::as_i64).collect();
        let off: Vec<usize> = t["data_offsets"].as_array().context("offsets")?.iter().filter_map(|v| v.as_u64().map(|x| x as usize)).collect();
        let raw = &body[off[0]..off[1]];
        let data = match t["dtype"].as_str().unwrap_or("") {
            "F32" => Data::F32(raw.chunks_exact(4).map(|c| f32::from_le_bytes(c.try_into().unwrap())).collect()),
            "I64" => Data::I64(raw.chunks_exact(8).map(|c| i64::from_le_bytes(c.try_into().unwrap())).collect()),
            other => bail!("{}: tensor {name} is {other}", file.display()),
        };
        out.insert(name.clone(), (shape, data));
    }
    Ok(out)
}

/// `src` (shape `ss`) copied into the leading corner of `dst` (shape `ds`), as pocket_tts_onnx.py's _adapt_state_tensor.
fn copy_corner<T: Copy>(src: &[T], ss: &[i64], dst: &mut [T], ds: &[i64]) {
    fn strides(s: &[i64]) -> Vec<usize> {
        let mut st = vec![1usize; s.len()];
        for i in (0..s.len().saturating_sub(1)).rev() {
            st[i] = st[i + 1] * s[i + 1] as usize;
        }
        st
    }
    let (sst, dst_st) = (strides(ss), strides(ds));
    let common: Vec<usize> = ss.iter().zip(ds).map(|(a, b)| (*a).min(*b) as usize).collect();
    fn rec<T: Copy>(d: usize, so: usize, doff: usize, c: &[usize], sst: &[usize], dstst: &[usize], src: &[T], dst: &mut [T]) {
        if d == c.len() - 1 {
            dst[doff..doff + c[d]].copy_from_slice(&src[so..so + c[d]]);
            return;
        }
        for i in 0..c[d] {
            rec(d + 1, so + i * sst[d], doff + i * dstst[d], c, sst, dstst, src, dst);
        }
    }
    if !common.is_empty() && common.iter().all(|&c| c > 0) {
        rec(0, 0, 0, &common, &sst, &dst_st, src, dst);
    }
}

/// Gaussian noise for the flow head: xorshift64* and Box-Muller.
struct Noise(u64);

impl Noise {
    fn next_f32(&mut self) -> f32 {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        ((self.0.wrapping_mul(0x2545_F491_4F6C_DD1D) >> 40) as f32 + 0.5) / (1u64 << 24) as f32
    }

    fn normal(&mut self, std: f32) -> f32 {
        let (u, v) = (self.next_f32(), self.next_f32());
        (-2.0 * u.ln()).sqrt() * (std::f32::consts::TAU * v).cos() * std
    }
}

/// What a line's `step` gave.
#[derive(Debug)]
pub enum Out {
    /// a sentence starts: its text as Pocket says it (one line, capitalised, a final stop)
    Begin(String),
    /// mono samples at sample_rate(), up to CHUNK_FRAMES frames (160 ms)
    Audio(Vec<f32>),
    /// the sentence ended
    End,
}

/// A line being said (Pocket::begin): its sentences still to say, the one being said, and its noise.
pub struct Gen {
    base: Vec<Init>,
    sentences: std::collections::VecDeque<(String, Vec<i64>, usize)>,
    cur: Option<Sentence>,
    noise: Noise,
    std: f32,
}

struct Sentence {
    state: Vec<(String, DynValue)>,
    mimi: Vec<(String, DynValue)>,
    curr: Vec<f32>,
    step: usize,
    limit: usize,
    eos_step: Option<usize>,
    frames_after_eos: usize,
    ending: bool,
}

// the line is said on the speech worker, one thread: its ort values only move with it
unsafe impl Send for Gen {}

struct Model {
    session: Session,
    inputs: Vec<String>,
    outputs: Vec<String>,
}

impl Model {
    fn load(file: &Path) -> anyhow::Result<Self> {
        let session = raea_sys_ml::session(file)?;
        let inputs = session.inputs().iter().map(|o| o.name().to_string()).collect();
        let outputs = session.outputs().iter().map(|o| o.name().to_string()).collect();
        Ok(Model { session, inputs, outputs })
    }

    /// Runs with `named` inputs plus `state` (by input name); returns every output in order. The state's outputs
    /// replace it (the outputs from `offset` on are the state, in manifest order).
    fn run(&mut self, named: Vec<(&str, DynValue)>, state: &mut [(String, DynValue)], offset: usize) -> anyhow::Result<Vec<DynValue>> {
        let mut inputs: Vec<(Cow<str>, SessionInputValue)> = named.into_iter().map(|(k, v)| (Cow::Owned(k.to_string()), v.into())).collect();
        for (name, v) in state.iter() {
            inputs.push((Cow::Owned(name.clone()), SessionInputValue::from(v)));
        }
        let mut out = self.session.run(inputs)?;
        let mut values = Vec::with_capacity(self.outputs.len());
        for name in &self.outputs {
            values.push(out.remove(name).ok_or_else(|| anyhow!("output {name} missing"))?);
        }
        drop(out);
        let rest = values.split_off(offset);
        for ((_, slot), v) in state.iter_mut().zip(rest) {
            *slot = v;
        }
        Ok(values)
    }
}

/// The executable that encodes a clip voice in a child process (`EXE --encode-voice NAME`); unset, it runs in-process.
pub static ENCODER: std::sync::OnceLock<std::path::PathBuf> = std::sync::OnceLock::new();

pub struct Pocket {
    tokenizer: tokenizers::Tokenizer,
    text_conditioner: Model,
    flow_main: Model,
    flow_head: Model,
    mimi: Model,
    /// Loaded on the first clip voice.
    mimi_encoder: Option<Model>,
    dir: std::path::PathBuf,
    flow_manifest: Vec<(Init, String, String)>,
    mimi_manifest: Vec<Init>,
    /// Each voice's flow-LM state after its prompt, loaded once.
    voices: HashMap<String, Vec<Init>>,
    sample_rate: u32,
    samples_per_frame: usize,
    frame_rate: f64,
    latent_dim: usize,
    conditioning_dim: usize,
    max_token_per_chunk: usize,
    eos_tokens: Vec<u32>,
    fallback_tokens: Vec<u32>,
    noise: Noise,
}

mod run;

/// pocket_tts_onnx.py's _prepare_text_prompt: one line, a capital first letter, a final period after a word; and how
/// many frames to keep after EOS (3 for four words or fewer, else 1; the caller adds 2).
fn prepare_text_prompt(text: &str) -> (String, usize) {
    let mut t = text.trim().replace(['\n', '\r'], " ").replace("  ", " ");
    if t.is_empty() {
        return (t, 1);
    }
    let guess = if t.split_whitespace().count() <= 4 { 3 } else { 1 };
    let mut chars = t.chars();
    if let Some(first) = chars.next().filter(|c| !c.is_uppercase()) {
        t = first.to_uppercase().chain(chars).collect();
    }
    if t.chars().last().is_some_and(char::is_alphanumeric) {
        t.push('.');
    }
    (t, guess)
}

/// A WAV (16-bit PCM or 32-bit float, any channels and rate) as mono samples at `rate` (linear resampling).
fn read_wav(path: &Path, rate: u32) -> anyhow::Result<Vec<f32>> {
    let b = std::fs::read(path).with_context(|| path.display().to_string())?;
    anyhow::ensure!(b.len() > 12 && &b[..4] == b"RIFF" && &b[8..12] == b"WAVE", "{}: not a WAV", path.display());
    let (mut fmt, mut data) = (None, None);
    let mut i = 12;
    while i + 8 <= b.len() {
        let len = u32::from_le_bytes(b[i + 4..i + 8].try_into()?) as usize;
        let body = &b[i + 8..(i + 8 + len).min(b.len())];
        match &b[i..i + 4] {
            b"fmt " => fmt = Some(body),
            b"data" => data = Some(body),
            _ => {}
        }
        i += 8 + len + (len & 1);
    }
    let (fmt, data) = (fmt.context("WAV without fmt")?, data.context("WAV without data")?);
    let format = u16::from_le_bytes([fmt[0], fmt[1]]);
    let channels = u16::from_le_bytes([fmt[2], fmt[3]]).max(1) as usize;
    let src_rate = u32::from_le_bytes(fmt[4..8].try_into()?);
    let bits = u16::from_le_bytes([fmt[14], fmt[15]]);
    let samples: Vec<f32> = match (format, bits) {
        (1, 16) | (0xFFFE, 16) => data.chunks_exact(2).map(|c| i16::from_le_bytes([c[0], c[1]]) as f32 / 32768.0).collect(),
        (3, 32) | (0xFFFE, 32) => data.chunks_exact(4).map(|c| f32::from_le_bytes(c.try_into().unwrap())).collect(),
        _ => bail!("{}: WAV format {format} with {bits} bits (want 16-bit PCM or 32-bit float)", path.display()),
    };
    let mono: Vec<f32> = samples.chunks_exact(channels).map(|c| c.iter().sum::<f32>() / channels as f32).collect();
    if src_rate == rate || mono.is_empty() {
        return Ok(mono);
    }
    let n = (mono.len() as u64 * rate as u64 / src_rate as u64) as usize;
    let step = src_rate as f64 / rate as f64;
    Ok((0..n)
        .map(|k| {
            let x = k as f64 * step;
            let (j, f) = (x as usize, (x.fract()) as f32);
            let a = mono[j.min(mono.len() - 1)];
            let b = mono[(j + 1).min(mono.len() - 1)];
            a + (b - a) * f
        })
        .collect())
}

/// The clip without the silence at either end: 20 ms windows quieter than 40 dB under the loudest are dropped, with
/// 100 ms kept on each side.
fn trim_silence(audio: &[f32], rate: usize) -> Vec<f32> {
    let w = rate / 50;
    let rms: Vec<f32> = audio.chunks(w).map(|c| (c.iter().map(|x| x * x).sum::<f32>() / c.len() as f32).sqrt()).collect();
    let peak = rms.iter().cloned().fold(0.0f32, f32::max);
    if peak <= 0.0 {
        return Vec::new();
    }
    let floor = peak * 0.01;
    let (Some(first), Some(last)) = (rms.iter().position(|&r| r > floor), rms.iter().rposition(|&r| r > floor)) else {
        return Vec::new();
    };
    let pad = rate / 10;
    let start = (first * w).saturating_sub(pad);
    let end = ((last + 1) * w + pad).min(audio.len());
    audio[start..end].to_vec()
}

/// A float32 .npy's values (bos_before_voice.npy).
fn read_npy_f32(path: &Path) -> anyhow::Result<Vec<f32>> {
    let b = std::fs::read(path).with_context(|| path.display().to_string())?;
    anyhow::ensure!(b.len() > 10 && &b[1..6] == b"NUMPY", "{}: not a .npy", path.display());
    let (header_len, start) = if b[6] == 1 {
        (u16::from_le_bytes([b[8], b[9]]) as usize, 10)
    } else {
        (u32::from_le_bytes(b[8..12].try_into()?) as usize, 12)
    };
    let header = String::from_utf8_lossy(&b[start..start + header_len]);
    anyhow::ensure!(header.contains("<f4") && header.contains("'fortran_order': False"), "{}: want little-endian float32, C order ({header})", path.display());
    Ok(b[start + header_len..].chunks_exact(4).map(|c| f32::from_le_bytes(c.try_into().unwrap())).collect())
}
