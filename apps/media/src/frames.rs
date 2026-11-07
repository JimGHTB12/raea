//! `media encode`: raw RGBA frames on stdin into an MP4 (docs/rune.feature R8a: a program that draws its own frames,
//! raea-rune-host's UI as video, hands them to raea's encoder without linking it). raea's own build signs the file and
//! marks its frames like every raea export; the public engine writes it unsigned, and the report's `credentials` says
//! which (docs/targets.feature T1e). A sound file is laid under the picture from 0 s, cut to the picture's length.

use std::io::Read;

use super::*;

/// Frames of `size` at `fps` from stdin, w x h x 4 bytes each, until it closes, into `out`.
pub fn frames(out: &Path, size: (u32, u32), fps: f64, quality: Option<f32>, audio: Option<&Path>) -> R<Value> {
    let (w, h) = size;
    if w == 0 || h == 0 || w % 2 == 1 || h % 2 == 1 {
        return Err(format!("--size {w}x{h}: an even width and height (H.264 halves the colour planes)"));
    }
    if let Some(d) = out.parent().filter(|d| !d.as_os_str().is_empty()) {
        std::fs::create_dir_all(d).map_err(|e| format!("{}: {e}", d.display()))?;
    }
    let kbps = ((w as f64 * h as f64 * fps * 0.1) / 1000.0).clamp(300.0, 20000.0) as u32;
    let mut enc = backend()?.encoder(out, &EncodeOpts { width: w, height: h, fps, kbps, quality, nv12: false, realtime: false })?;
    let mut buf = vec![0u8; w as usize * h as usize * 4];
    let mut input = std::io::stdin().lock();
    let mut frames = 0usize;
    loop {
        let mut got = 0;
        while got < buf.len() {
            match input.read(&mut buf[got..]) {
                Ok(0) => break,
                Ok(n) => got += n,
                Err(e) if e.kind() == std::io::ErrorKind::Interrupted => {}
                Err(e) => return Err(format!("stdin: {e}")),
            }
        }
        if got == 0 {
            break;
        }
        if got < buf.len() {
            return Err(format!("stdin ended inside frame {frames}: {got} of its {} bytes ({w}x{h} RGBA)", buf.len()));
        }
        enc.push(&buf)?;
        frames += 1;
    }
    if frames == 0 {
        return Err("no frames on stdin".into());
    }
    let secs = frames as f64 / fps;
    let sound = match audio {
        Some(p) => {
            let a = soft()?.decode_audio(p).map_err(|e| format!("{}: {e}", p.display()))?;
            let a = raea_io_media::soft_resample(&a, raea_io_media::audio::RATE);
            let n = (secs * raea_io_media::audio::RATE as f64).round() as usize;
            let channels = a.channels.into_iter().map(|mut c| {
                c.resize(n, 0.0);
                c
            });
            Some(Audio { rate: raea_io_media::audio::RATE, channels: channels.collect() })
        }
        None => None,
    };
    let stats = enc.finish(sound.as_ref())?;
    Ok(json!({"wrote": out, "frames": frames, "seconds": round3(secs), "width": w, "height": h, "fps": round3(fps), "bytes": stats.bytes, "sound": sound.is_some(), "credentials": if raea_io_media::SIGNS { "signed" } else { "unsigned" }}))
}
