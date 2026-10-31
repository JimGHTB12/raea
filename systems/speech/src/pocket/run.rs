//! Pocket's generation: a voice's prompt, then sentences streamed into audio chunks.

use super::*;

impl Pocket {
    pub const SPEC: raea_sys_ml::models::Spec = raea_sys_ml::models::Spec {
        name: "pocket-tts",
        files: &[
            (BUNDLE, "onnx/english_2026-04/bundle.json", "bundle.json"),
            (BUNDLE, "onnx/english_2026-04/text_conditioner.onnx", "text_conditioner.onnx"),
            (BUNDLE, "onnx/english_2026-04/flow_lm_main_int8.onnx", "flow_lm_main_int8.onnx"),
            (BUNDLE, "onnx/english_2026-04/flow_lm_flow_int8.onnx", "flow_lm_flow_int8.onnx"),
            (BUNDLE, "onnx/english_2026-04/mimi_decoder_int8.onnx", "mimi_decoder_int8.onnx"),
            // voice cloning: the encoder for clips, and the BOS embedding that goes before a clip's prompt
            (BUNDLE, "onnx/english_2026-04/mimi_encoder.onnx", "mimi_encoder.onnx"),
            (BUNDLE, "onnx/english_2026-04/bos_before_voice.npy", "bos_before_voice.npy"),
            ("mlx-community/pocket-tts", "tokenizer.json", "tokenizer.json"),
            (VOICE_REPO, "languages/english_2026-04/embeddings/alba.safetensors", "voice-alba.safetensors"),
            (VOICE_REPO, "languages/english_2026-04/embeddings/marius.safetensors", "voice-marius.safetensors"),
            (VOICE_REPO, "languages/english_2026-04/embeddings/javert.safetensors", "voice-javert.safetensors"),
            (VOICE_REPO, "languages/english_2026-04/embeddings/jean.safetensors", "voice-jean.safetensors"),
            (VOICE_REPO, "languages/english_2026-04/embeddings/fantine.safetensors", "voice-fantine.safetensors"),
            (VOICE_REPO, "languages/english_2026-04/embeddings/cosette.safetensors", "voice-cosette.safetensors"),
            (VOICE_REPO, "languages/english_2026-04/embeddings/eponine.safetensors", "voice-eponine.safetensors"),
            (VOICE_REPO, "languages/english_2026-04/embeddings/azelma.safetensors", "voice-azelma.safetensors"),
        ],
    };
    pub const VOICES: &[&str] = &["alba", "marius", "javert", "jean", "fantine", "cosette", "eponine", "azelma"];

    pub fn load(dir: &Path) -> anyhow::Result<Self> {
        let bundle: Value = serde_json::from_slice(&std::fs::read(dir.join("bundle.json"))?)?;
        let tokenizer = tokenizers::Tokenizer::from_file(dir.join("tokenizer.json")).map_err(|e| anyhow!("tokenizer: {e}"))?;
        let flow_manifest = manifest(&bundle["flow_lm_state_manifest"])?;
        let mimi_manifest = manifest(&bundle["mimi_state_manifest"])?.into_iter().map(|(i, _, _)| i).collect();
        let num = |k: &str| bundle[k].as_f64().ok_or_else(|| anyhow!("bundle.json: {k}?"));
        let encode = |s: &str| -> anyhow::Result<Vec<u32>> {
            Ok(tokenizer.encode(s, false).map_err(|e| anyhow!("tokenize: {e}"))?.get_ids().to_vec())
        };
        // the boundaries pocket_tts_onnx.py splits on: the tokens of ".!...?" and ",;:" after the leading "▁"
        let eos_tokens = encode(".!...?")?[1..].to_vec();
        let fallback_tokens = encode(",;:")?[1..].to_vec();
        let seed = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos() as u64).unwrap_or(1) | 1;
        Ok(Pocket {
            text_conditioner: Model::load(&dir.join("text_conditioner.onnx"))?,
            flow_main: Model::load(&dir.join("flow_lm_main_int8.onnx"))?,
            flow_head: Model::load(&dir.join("flow_lm_flow_int8.onnx"))?,
            mimi: Model::load(&dir.join("mimi_decoder_int8.onnx"))?,
            mimi_encoder: None,
            dir: dir.to_path_buf(),
            flow_manifest,
            mimi_manifest,
            voices: HashMap::new(),
            sample_rate: num("sample_rate")? as u32,
            samples_per_frame: num("samples_per_frame")? as usize,
            frame_rate: num("frame_rate")?,
            latent_dim: num("latent_dim")? as usize,
            conditioning_dim: num("conditioning_dim")? as usize,
            max_token_per_chunk: bundle["max_token_per_chunk"].as_u64().unwrap_or(50) as usize,
            eos_tokens,
            fallback_tokens,
            tokenizer,
            noise: Noise(seed),
        })
    }

    /// Where clip voices live: ~/.raea-feed/voices/<name>.wav (raea_sys_ml::feed::person).
    pub fn voice_dir() -> std::path::PathBuf {
        raea_sys_ml::feed::person().join("voices")
    }

    /// Every voice: the built-ins, then each WAV in voice_dir() by file name.
    pub fn voices() -> Vec<String> {
        let mut out: Vec<String> = Self::VOICES.iter().map(|v| v.to_string()).collect();
        let mut clips: Vec<String> = std::fs::read_dir(Self::voice_dir())
            .into_iter()
            .flatten()
            .flatten()
            .map(|e| e.path())
            .filter(|p| p.extension().is_some_and(|x| x.eq_ignore_ascii_case("wav")))
            .filter_map(|p| p.file_stem().map(|s| s.to_string_lossy().into_owned()))
            .filter(|n| !Self::VOICES.contains(&n.as_str()))
            .collect();
        clips.sort();
        out.extend(clips);
        out
    }

    /// The voice to use when none is chosen: the founder's clip "watts" when it's there, otherwise alba.
    pub fn default_voice() -> String {
        if Self::voice_dir().join("watts.wav").exists() { "watts".into() } else { "alba".into() }
    }

    /// Output rate (24 kHz).
    /// Encode a clip voice's state to disk (the child process's whole job: `EXE --encode-voice NAME`).
    pub fn encode_voice(&mut self, voice: &str) -> anyhow::Result<()> {
        self.voice_state(voice).map(|_| ())
    }

    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    /// `text` in `voice`, streamed: `on_audio` gets mono samples at sample_rate() as frames decode. Returns early,
    /// without error, once `cancel` is set (barge-in). Each call draws fresh noise (void's voice: a line said twice
    /// isn't said the same way twice); `begin` with a seed says it the same way every time.
    pub fn speak(&mut self, text: &str, voice: &str, cancel: &AtomicBool, on_audio: &mut dyn FnMut(&[f32])) -> anyhow::Result<()> {
        let seed = ((self.noise.next_f32().to_bits() as u64) << 32) | self.noise.next_f32().to_bits() as u64;
        let mut g = self.begin(text, voice, seed, TEMPERATURE)?;
        while !cancel.load(Ordering::Relaxed) {
            match self.step(&mut g)? {
                Some(Out::Audio(a)) => on_audio(&a),
                Some(_) => {}
                None => break,
            }
        }
        Ok(())
    }

    /// A line to be said, one `step` at a time: the voice's state, the text split as Pocket splits it, and noise from
    /// `seed`. The same text, voice, seed and temperature give the same samples every time on one machine.
    pub fn begin(&mut self, text: &str, voice: &str, seed: u64, temperature: f32) -> anyhow::Result<Gen> {
        let base = self.voice_state(voice)?;
        let mut sentences = std::collections::VecDeque::new();
        if !text.trim().is_empty() {
            for chunk in self.split_into_best_sentences(text)? {
                let (prepared, guess) = prepare_text_prompt(&chunk);
                if prepared.is_empty() {
                    continue;
                }
                let ids: Vec<i64> = self.encode(&prepared)?.into_iter().map(i64::from).collect();
                sentences.push_back((prepared, ids, guess + 2));
            }
        }
        Ok(Gen { base, sentences, cur: None, noise: Noise(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1), std: temperature.max(0.0).sqrt() })
    }

    /// The line's next piece: a sentence beginning (its text as Pocket says it), up to CHUNK_FRAMES frames of audio, or
    /// a sentence ending; None when the line is done.
    pub fn step(&mut self, g: &mut Gen) -> anyhow::Result<Option<Out>> {
        let (ld, cd) = (self.latent_dim, self.conditioning_dim);
        let Some(s) = g.cur.as_mut() else {
            let Some((text, ids, frames_after_eos)) = g.sentences.pop_front() else { return Ok(None) };
            let mut state: Vec<(String, DynValue)> = g.base.iter().map(|i| Ok((i.name.clone(), i.value()?))).collect::<anyhow::Result<_>>()?;
            let mimi: Vec<(String, DynValue)> = self.mimi_manifest.iter().map(|i| Ok((i.name.clone(), i.value()?))).collect::<anyhow::Result<_>>()?;
            let n_tokens = ids.len();
            let tc_input = self.text_conditioner.inputs[0].clone();
            let tc = self.text_conditioner.run(vec![(&tc_input, Tensor::from_array(([1usize, n_tokens], ids))?.into_dyn())], &mut [], 1)?;
            let (shape, emb) = tc[0].try_extract_tensor::<f32>()?;
            let m = if shape.len() == 2 { shape[0] as usize } else { shape[1] as usize };
            let text_embeddings = Tensor::from_array(([1usize, m, cd], emb.to_vec()))?.into_dyn();
            self.flow_main.run(
                vec![("sequence", Tensor::from_array(([1usize, 0, ld], Vec::<f32>::new()))?.into_dyn()), ("text_embeddings", text_embeddings)],
                &mut state,
                2,
            )?;
            let limit = ((n_tokens as f64 / 3.0 + 2.0) * self.frame_rate).ceil() as usize;
            g.cur = Some(Sentence { state, mimi, curr: vec![f32::NAN; ld], step: 0, limit, eos_step: None, frames_after_eos, ending: false });
            return Ok(Some(Out::Begin(text)));
        };
        if s.ending {
            g.cur = None;
            return Ok(Some(Out::End));
        }
        let mut pending: Vec<f32> = Vec::with_capacity(CHUNK_FRAMES * ld);
        let dt = 1.0 / LSD_STEPS as f32;
        while pending.len() < CHUNK_FRAMES * ld {
            if s.step >= s.limit || s.eos_step.is_some_and(|e| s.step >= e + s.frames_after_eos) {
                s.ending = true;
                break;
            }
            let out = self.flow_main.run(
                vec![
                    ("sequence", Tensor::from_array(([1usize, 1, ld], s.curr.clone()))?.into_dyn()),
                    ("text_embeddings", Tensor::from_array(([1usize, 0, cd], Vec::<f32>::new()))?.into_dyn()),
                ],
                &mut s.state,
                2,
            )?;
            let (_, eos) = out[1].try_extract_tensor::<f32>()?;
            if s.eos_step.is_none() && eos[0] > EOS_LOGIT {
                s.eos_step = Some(s.step);
            }
            if s.eos_step.is_some_and(|e| s.step >= e + s.frames_after_eos) {
                s.ending = true;
                break;
            }
            let (cshape, c) = out[0].try_extract_tensor::<f32>()?;
            let cshape: Vec<i64> = cshape.iter().copied().collect();
            let c = c.to_vec();
            let mut x: Vec<f32> = (0..ld).map(|_| g.noise.normal(g.std)).collect();
            for j in 0..LSD_STEPS {
                let sj = j as f32 / LSD_STEPS as f32;
                let flow = self.flow_head.run(
                    vec![
                        ("c", Tensor::from_array((cshape.clone(), c.clone()))?.into_dyn()),
                        ("s", Tensor::from_array(([1usize, 1], vec![sj]))?.into_dyn()),
                        ("t", Tensor::from_array(([1usize, 1], vec![sj + dt]))?.into_dyn()),
                        ("x", Tensor::from_array(([1usize, ld], x.clone()))?.into_dyn()),
                    ],
                    &mut [],
                    1,
                )?;
                let (_, f) = flow[0].try_extract_tensor::<f32>()?;
                for (xi, fi) in x.iter_mut().zip(f) {
                    *xi += fi * dt;
                }
            }
            pending.extend_from_slice(&x);
            s.curr = x;
            s.step += 1;
        }
        if pending.is_empty() {
            g.cur = None;
            return Ok(Some(Out::End));
        }
        let frames = pending.len() / ld;
        let out = self.mimi.run(vec![("latent", Tensor::from_array(([1usize, frames, ld], pending))?.into_dyn())], &mut s.mimi, 1)?;
        let (_, audio) = out[0].try_extract_tensor::<f32>()?;
        debug_assert_eq!(audio.len(), frames * self.samples_per_frame);
        Ok(Some(Out::Audio(audio.to_vec())))
    }

    pub(crate) fn encode(&self, s: &str) -> anyhow::Result<Vec<u32>> {
        Ok(self.tokenizer.encode(s, false).map_err(|e| anyhow!("tokenize: {e}"))?.get_ids().to_vec())
    }

    pub(crate) fn decode(&self, ids: &[u32]) -> anyhow::Result<String> {
        self.tokenizer.decode(ids, true).map_err(|e| anyhow!("detokenize: {e}"))
    }

    /// The flow-LM state after `voice`'s prompt: its saved KV caches placed into the manifest's full-size tensors
    /// (pocket_tts_onnx.py's _state_from_model_state).
    pub(crate) fn voice_state(&mut self, voice: &str) -> anyhow::Result<Vec<Init>> {
        if let Some(s) = self.voices.get(voice) {
            return Ok(s.clone());
        }
        let file = self.dir.join(format!("voice-{voice}.safetensors"));
        if !Self::VOICES.contains(&voice) || !file.exists() {
            // a voice is a clip in voice_dir() by name, or a WAV anywhere by its path (a game's own narrator clip)
            let clip = if voice.ends_with(".wav") && Path::new(voice).is_absolute() { std::path::PathBuf::from(voice) } else { Self::voice_dir().join(format!("{voice}.wav")) };
            let name = clip.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_else(|| voice.to_string());
            if clip.exists() {
                // the clip's state is saved next to it after the first encoding (the encoder's activations over a
                // 30 s clip peak near 3 GB, so it runs once per clip, not once per launch)
                let meta = std::fs::metadata(&clip)?;
                let stamp = meta.modified().ok().and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok()).map(|d| d.as_secs()).unwrap_or(0);
                let saved = Self::voice_dir().join(".state").join(format!("{name}-{}-{stamp}.safetensors", meta.len()));
                let state = match saved.exists().then(|| safetensors(&saved)).transpose()? {
                    Some(s) => self.state_from_saved(voice, &s)?,
                    // the encoding's ~3 GB peak runs in a child process when the app names one (ENCODER), so the
                    // app's own footprint never holds it (memguard kills a job over 6 GB)
                    None if ENCODER.get().is_some() => {
                        let exe = ENCODER.get().unwrap();
                        let st = std::process::Command::new(exe).args(["--encode-voice", voice]).status()?;
                        anyhow::ensure!(st.success() && saved.exists(), "encoding {voice}'s clip failed ({st})");
                        self.state_from_saved(voice, &safetensors(&saved)?)?
                    }
                    None => {
                        let state = self.clip_state(&clip)?;
                        if let Err(e) = self.save_state(&state, &saved) {
                            eprintln!("pocket: couldn't save {voice}'s state: {e}");
                        }
                        state
                    }
                };
                self.voices.insert(voice.to_string(), state.clone());
                return Ok(state);
            }
            bail!("no Pocket voice {voice:?} (voices: {})", Self::voices().join(", "));
        }
        let state = self.state_from_saved(voice, &safetensors(&file)?)?;
        self.voices.insert(voice.to_string(), state.clone());
        Ok(state)
    }

    /// A saved voice state (Kyutai's format: "<module>/cache" sliced to the prompt, "<module>/offset") placed into the
    /// manifest's full-size tensors.
    pub(crate) fn state_from_saved(&self, voice: &str, saved: &HashMap<String, (Vec<i64>, Data)>) -> anyhow::Result<Vec<Init>> {
        let mut state = Vec::new();
        for (init, module, key) in &self.flow_manifest {
            let mut init = init.clone();
            let found = saved.get(&format!("{module}/{key}")).cloned().or_else(|| {
                // a step the file doesn't hold is its offset (the prompt's length)
                (key == "step").then(|| saved.get(&format!("{module}/offset")).cloned()).flatten()
            });
            if let Some((shape, data)) = found {
                match (&mut init.data, data) {
                    (Data::F32(dst), Data::F32(src)) => {
                        if src.len() == dst.len() {
                            *dst = src;
                        } else {
                            copy_corner(&src, &shape, dst, &init.shape);
                        }
                    }
                    (Data::I64(dst), Data::I64(src)) if src.len() == dst.len() => *dst = src,
                    (Data::I64(dst), Data::I64(src)) => copy_corner(&src, &shape, dst, &init.shape),
                    _ => bail!("{voice}: {module}/{key} has the wrong type"),
                }
            }
            state.push(init);
        }
        Ok(state)
    }

    /// A clip's voice: its audio (24 kHz mono, silence trimmed, at most 30 s) through the Mimi encoder, the BOS
    /// embedding before it, and the flow LM run over that prompt from an empty state.
    pub(crate) fn clip_state(&mut self, clip: &Path) -> anyhow::Result<Vec<Init>> {
        let audio = trim_silence(&read_wav(clip, self.sample_rate)?, self.sample_rate as usize);
        let audio: Vec<f32> = audio.into_iter().take(MAX_PROMPT_SECONDS * self.sample_rate as usize).collect();
        anyhow::ensure!(audio.len() > self.samples_per_frame, "{}: no speech in the clip", clip.display());
        if self.mimi_encoder.is_none() {
            self.mimi_encoder = Some(Model::load(&self.dir.join("mimi_encoder.onnx"))?);
        }
        let encoder = self.mimi_encoder.as_mut().unwrap();
        let n = audio.len();
        let out = encoder.run(vec![("audio", Tensor::from_array(([1usize, 1, n], audio))?.into_dyn())], &mut [], 1)?;
        let (shape, emb) = out[0].try_extract_tensor::<f32>()?;
        let cd = self.conditioning_dim;
        anyhow::ensure!(shape.last() == Some(&(cd as i64)), "mimi encoder gave {shape:?}");
        let mut prompt = read_npy_f32(&self.dir.join("bos_before_voice.npy"))?;
        prompt.extend_from_slice(emb);
        let frames = prompt.len() / cd;
        let mut state: Vec<(String, DynValue)> =
            self.flow_manifest.iter().map(|(i, _, _)| Ok((i.name.clone(), i.value()?))).collect::<anyhow::Result<_>>()?;
        self.flow_main.run(
            vec![
                ("sequence", Tensor::from_array(([1usize, 0, self.latent_dim], Vec::<f32>::new()))?.into_dyn()),
                ("text_embeddings", Tensor::from_array(([1usize, frames, cd], prompt))?.into_dyn()),
            ],
            &mut state,
            2,
        )?;
        state.into_iter().map(|(name, v)| Init::from_value(name, &v)).collect()
    }

    /// A voice state written as Kyutai's voice files are: each layer's KV cache cut to the prompt's length, and its
    /// offset.
    pub(crate) fn save_state(&self, state: &[Init], file: &Path) -> anyhow::Result<()> {
        let mut tensors: Vec<(String, Vec<i64>, Data)> = Vec::new();
        for (init, (_, module, key)) in state.iter().zip(&self.flow_manifest) {
            if key != "step" {
                continue;
            }
            let Data::I64(step) = &init.data else { bail!("{module}/step isn't int64") };
            let frames = step[0];
            tensors.push((format!("{module}/offset"), vec![1], Data::I64(vec![frames])));
            let cache = state.iter().zip(&self.flow_manifest).find(|(_, (_, m, k))| m == module && k == "cache").map(|(i, _)| i);
            let Some(Init { shape, data: Data::F32(full), .. }) = cache else { bail!("{module}/cache missing") };
            let mut cut_shape = shape.clone();
            cut_shape[2] = frames.min(shape[2]);
            let mut cut = vec![0f32; cut_shape.iter().map(|&d| d as usize).product()];
            copy_corner(full, shape, &mut cut, &cut_shape);
            tensors.push((format!("{module}/cache"), cut_shape, Data::F32(cut)));
        }
        let mut header = serde_json::Map::new();
        let mut body: Vec<u8> = Vec::new();
        for (name, shape, data) in &tensors {
            let start = body.len();
            let dtype = match data {
                Data::F32(v) => {
                    v.iter().for_each(|x| body.extend_from_slice(&x.to_le_bytes()));
                    "F32"
                }
                Data::I64(v) => {
                    v.iter().for_each(|x| body.extend_from_slice(&x.to_le_bytes()));
                    "I64"
                }
                Data::Bool(_) => bail!("bool in a flow state"),
            };
            header.insert(name.clone(), serde_json::json!({"dtype": dtype, "shape": shape, "data_offsets": [start, body.len()]}));
        }
        let mut head = serde_json::to_vec(&Value::Object(header))?;
        while head.len() % 8 != 0 {
            head.push(b' ');
        }
        std::fs::create_dir_all(file.parent().context("state path")?)?;
        let mut bytes = (head.len() as u64).to_le_bytes().to_vec();
        bytes.extend_from_slice(&head);
        bytes.extend_from_slice(&body);
        let part = file.with_extension("part");
        std::fs::write(&part, bytes)?;
        std::fs::rename(part, file)?;
        Ok(())
    }

    /// pocket_tts_onnx.py's _split_into_best_sentences: split at sentence ends (then at commas when a sentence is longer
    /// than max_token_per_chunk tokens), and pack the pieces into chunks of at most that many tokens.
    pub(crate) fn split_into_best_sentences(&self, text: &str) -> anyhow::Result<Vec<String>> {
        let (prepared, _) = prepare_text_prompt(text);
        let tokens = self.encode(prepared.trim())?;
        let segments = self.segments(&tokens, &self.eos_tokens)?;
        let mut refined = Vec::new();
        for (count, seg) in segments {
            if count <= self.max_token_per_chunk {
                refined.push((count, seg));
                continue;
            }
            let sub = self.encode(seg.trim())?;
            let subs = self.segments(&sub, &self.fallback_tokens)?;
            if subs.len() > 1 {
                refined.extend(subs);
            } else {
                refined.push((count, seg));
            }
        }
        let mut chunks = Vec::new();
        let (mut current, mut current_count) = (String::new(), 0usize);
        for (count, seg) in refined {
            if current.is_empty() {
                current = seg;
                current_count = count;
            } else if current_count + count > self.max_token_per_chunk {
                chunks.push(current.trim().to_string());
                current = seg;
                current_count = count;
            } else {
                current.push(' ');
                current.push_str(&seg);
                current_count += count;
            }
        }
        if !current.is_empty() {
            chunks.push(current.trim().to_string());
        }
        Ok(chunks)
    }

    pub(crate) fn segments(&self, tokens: &[u32], boundary: &[u32]) -> anyhow::Result<Vec<(usize, String)>> {
        let mut idx = vec![0usize];
        let mut previous = false;
        for (i, t) in tokens.iter().enumerate() {
            if boundary.contains(t) {
                previous = true;
            } else {
                if previous {
                    idx.push(i);
                }
                previous = false;
            }
        }
        idx.push(tokens.len());
        idx.windows(2).map(|w| Ok((w[1] - w[0], self.decode(&tokens[w[0]..w[1]])?))).collect()
    }
}
