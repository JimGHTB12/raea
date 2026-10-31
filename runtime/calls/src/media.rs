//! mediabunny's coders (`raea/media`): VideoToolbox video, AudioToolbox and Symphonia audio, engine frames.

use serde_json::{Value, json};

use super::{Picture, R, State, ecs, f, s, u};

fn b64(v: &Value, k: &str) -> R<Vec<u8>> {
    use base64::Engine as _;
    match v.get(k).and_then(Value::as_str) {
        Some(s) => base64::engine::general_purpose::STANDARD.decode(s).map_err(|e| format!("{k}: {e}")),
        None => Ok(Vec::new()),
    }
}

fn enc64(b: &[u8]) -> String {
    use base64::Engine as _;
    base64::engine::general_purpose::STANDARD.encode(b)
}

/// mediabunny's transformation of a frame (VideoSampleTransformationDescription), on NV12: rotated clockwise, flipped
/// about the vertical axis, cropped (in the rotated picture), then resized to w x h by `fit` (fill stretches, contain
/// letterboxes in black, cover crops the middle). Each output pixel is mapped back to its source pixel.
fn transform_nv12(src: &Picture, rotation: u32, flip: bool, crop: Option<(f64, f64, f64, f64)>, w: u32, h: u32, fit: &str) -> Picture {
    let (sw, sh) = (src.width as f64, src.height as f64);
    // the rotated picture's size
    let (rw, rh) = if rotation % 180 == 0 { (sw, sh) } else { (sh, sw) };
    let (cx, cy, cw, ch) = crop.unwrap_or((0.0, 0.0, rw, rh));
    let (ow, oh) = (w as f64, h as f64);
    // the cropped picture placed in the output box
    let (sx, sy) = (ow / cw, oh / ch);
    let (kx, ky, ox, oy) = match fit {
        "contain" => {
            let k = sx.min(sy);
            (k, k, (ow - cw * k) / 2.0, (oh - ch * k) / 2.0)
        }
        "cover" => {
            let k = sx.max(sy);
            (k, k, (ow - cw * k) / 2.0, (oh - ch * k) / 2.0)
        }
        _ => (sx, sy, 0.0, 0.0),
    };
    // a point of the output (pixel centre) to a source point; None outside (letterbox)
    let back = |x: f64, y: f64| -> Option<(f64, f64)> {
        let (mut u, v) = ((x - ox) / kx + cx, (y - oy) / ky + cy);
        if u < cx || v < cy || u >= cx + cw || v >= cy + ch {
            return None;
        }
        if flip {
            u = rw - u;
        }
        // undo a clockwise rotation
        // (a source point (x, y) turned 90 clockwise lands at (sh - y, x) in the rotated picture, which is rw x rh = sh x sw)
        let (px, py) = match rotation % 360 {
            90 => (v, rw - u),
            180 => (rw - u, rh - v),
            270 => (rh - v, u),
            _ => (u, v),
        };
        Some((px.clamp(0.0, sw - 1e-6), py.clamp(0.0, sh - 1e-6)))
    };
    let (dw, dh) = (w as usize, h as usize);
    let (sw_i, sh_i) = (src.width as usize, src.height as usize);
    let mut out = vec![0u8; dw * dh + dw * (dh / 2)];
    for y in 0..dh {
        for x in 0..dw {
            out[y * dw + x] = match back(x as f64 + 0.5, y as f64 + 0.5) {
                Some((px, py)) => src.nv12[py as usize * sw_i + px as usize],
                None => 16,
            };
        }
    }
    let uv = dw * dh;
    for y in 0..dh / 2 {
        for x in 0..dw / 2 {
            let (u, v) = match back(x as f64 * 2.0 + 1.0, y as f64 * 2.0 + 1.0) {
                Some((px, py)) => {
                    let at = sw_i * sh_i + (py as usize / 2) * sw_i + (px as usize / 2) * 2;
                    (src.nv12[at], src.nv12[at + 1])
                }
                None => (128, 128),
            };
            out[uv + y * dw + x * 2] = u;
            out[uv + y * dw + x * 2 + 1] = v;
        }
    }
    Picture { width: w, height: h, nv12: out }
}

/// NV12 scaled (box-averaged when shrinking, nearest when growing), each plane on its own.
fn scale_nv12(src: &Picture, w: u32, h: u32) -> Picture {
    let (sw, sh, dw, dh) = (src.width as usize, src.height as usize, w as usize, h as usize);
    let mut out = vec![0u8; dw * dh + dw * (dh / 2)];
    let sample = |plane: &[u8], pw: usize, ph: usize, step: usize, ch: usize, x0: f64, x1: f64, y0: f64, y1: f64| -> u8 {
        let (xa, xb) = (x0.floor() as usize, (x1.ceil() as usize).max(x0.floor() as usize + 1).min(pw));
        let (ya, yb) = (y0.floor() as usize, (y1.ceil() as usize).max(y0.floor() as usize + 1).min(ph));
        let (mut s, mut n) = (0u32, 0u32);
        for y in ya..yb {
            for x in xa..xb {
                s += plane[y * pw * step + x * step + ch] as u32;
                n += 1;
            }
        }
        (s / n.max(1)) as u8
    };
    let (sy, suv) = src.nv12.split_at(sw * sh);
    for y in 0..dh {
        for x in 0..dw {
            let (x0, x1) = (x as f64 * sw as f64 / dw as f64, (x + 1) as f64 * sw as f64 / dw as f64);
            let (y0, y1) = (y as f64 * sh as f64 / dh as f64, (y + 1) as f64 * sh as f64 / dh as f64);
            out[y * dw + x] = sample(sy, sw, sh, 1, 0, x0, x1, y0, y1);
        }
    }
    let (cw, chh, scw, sch) = (dw / 2, dh / 2, sw / 2, sh / 2);
    for y in 0..chh {
        for x in 0..cw {
            let (x0, x1) = (x as f64 * scw as f64 / cw as f64, (x + 1) as f64 * scw as f64 / cw as f64);
            let (y0, y1) = (y as f64 * sch as f64 / chh as f64, (y + 1) as f64 * sch as f64 / chh as f64);
            for c in 0..2 {
                out[dw * dh + y * dw + x * 2 + c] = sample(suv, scw, sch, 2, c, x0, x1, y0, y1);
            }
        }
    }
    Picture { width: w, height: h, nv12: out }
}

pub(super) fn call(st: &mut State, name: &str, v: &Value) -> R<Value> {
    match name {
        // ---- media (mediabunny's coders, raea/media)
        "media.live" => {
            #[cfg(target_vendor = "apple")]
            let (vd, ve) = (st.vdecs.len(), st.vencs.len() + st.aencs.len());
            #[cfg(not(target_vendor = "apple"))]
            let (vd, ve) = (0usize, 0usize);
            Ok(json!({"vdecs": vd, "vencs": ve, "adecs": st.adecs.len(), "frames": st.pictures.len(), "solids": st.solids.len(), "worlds": ecs::live()}))
        }
        "media.size" => Ok(json!(std::fs::metadata(s(v, "path")?).map_err(|e| e.to_string())?.len())),
        "media.read" => {
            use std::io::{Read, Seek, SeekFrom};
            let (start, end) = (u(v, "start")?, u(v, "end")?);
            let mut f = std::fs::File::open(s(v, "path")?).map_err(|e| e.to_string())?;
            f.seek(SeekFrom::Start(start)).map_err(|e| e.to_string())?;
            let mut buf = vec![0u8; end.saturating_sub(start) as usize];
            let n = f.read(&mut buf).map_err(|e| e.to_string())?;
            buf.truncate(n);
            let mut rest = buf.len();
            while rest < (end - start) as usize {
                let mut more = vec![0u8; (end - start) as usize - rest];
                let k = f.read(&mut more).map_err(|e| e.to_string())?;
                if k == 0 {
                    break;
                }
                buf.extend_from_slice(&more[..k]);
                rest = buf.len();
            }
            Ok(json!(enc64(&buf)))
        }
        "media.write" => {
            use std::io::{Seek, SeekFrom, Write as _};
            let path = s(v, "path")?;
            let data = b64(v, "data")?;
            let mut f = std::fs::OpenOptions::new().create(true).write(true).truncate(v.get("truncate").and_then(Value::as_bool).unwrap_or(false)).open(path).map_err(|e| format!("{path}: {e}"))?;
            f.seek(SeekFrom::Start(v.get("at").and_then(Value::as_u64).unwrap_or(0))).map_err(|e| e.to_string())?;
            f.write_all(&data).map_err(|e| e.to_string())?;
            Ok(Value::Null)
        }
        #[cfg(target_vendor = "apple")]
        "vdec.new" => {
            let d = raea_io_media::apple::packets::PacketDecoder::new(s(v, "codec")?, &b64(v, "description")?)?;
            st.next += 1;
            let id = st.next;
            st.vdecs.insert(id, d);
            Ok(json!({"id": id}))
        }
        #[cfg(target_vendor = "apple")]
        "vdec.decode" | "vdec.flush" => {
            let id = u(v, "id")?;
            let frames = {
                let d = st.vdecs.get_mut(&id).ok_or("no such decoder")?;
                if name == "vdec.flush" { d.flush()? } else { d.decode(&b64(v, "data")?, f(v, "pts")?, f(v, "dur")?)? }
            };
            let mut out = Vec::new();
            for fr in frames {
                st.next += 1;
                let pid = st.next;
                out.push(json!({"frame": pid, "pts": fr.pts, "dur": fr.dur, "width": fr.width, "height": fr.height}));
                st.pictures.insert(pid, Picture { width: fr.width, height: fr.height, nv12: fr.nv12 });
            }
            Ok(Value::Array(out))
        }
        #[cfg(target_vendor = "apple")]
        "vdec.free" => {
            st.vdecs.remove(&u(v, "id")?);
            Ok(Value::Null)
        }
        #[cfg(target_vendor = "apple")]
        "venc.new" => {
            let e = raea_io_media::apple::packets::PacketEncoder::new(u(v, "width")? as u32, u(v, "height")? as u32, f(v, "fps")?, u(v, "kbps")? as u32)?;
            st.next += 1;
            let id = st.next;
            st.vencs.insert(id, e);
            Ok(json!({"id": id}))
        }
        #[cfg(target_vendor = "apple")]
        "venc.encode" | "venc.finish" => {
            let id = u(v, "id")?;
            let packets = if name == "venc.finish" {
                st.vencs.get_mut(&id).ok_or("no such encoder")?.finish()?
            } else {
                let pic = st.pictures.get(&u(v, "frame")?).ok_or("that frame is gone")?;
                let nv12 = pic.nv12.clone();
                st.vencs.get_mut(&id).ok_or("no such encoder")?.encode(&nv12, f(v, "pts")?, f(v, "dur")?)?
            };
            let e = st.vencs.get(&id).unwrap();
            let desc = e.description().map(|d| enc64(&d));
            if name == "venc.finish" {
                st.vencs.remove(&id);
            }
            Ok(json!({"packets": packets.iter().map(|p| json!({"data": enc64(&p.data), "key": p.key, "pts": p.pts, "dur": p.dur})).collect::<Vec<_>>(), "description": desc}))
        }
        // a heavy job in one native call: decode, scale, encode with sound (what `raea media trim|proxy` does, here
        // in-process so it runs the same in an app); {path, out, from?, to?, height?, kbps?}
        "media.transcode" => {
            let (path, out) = (s(v, "path")?, s(v, "out")?);
            let be = raea_io_media::backend(if cfg!(target_vendor = "apple") { "apple" } else { "soft" })?;
            let mut dec = be.open_video(std::path::Path::new(path))?;
            let info = dec.info().clone();
            let (from, to) = (v.get("from").and_then(Value::as_f64).unwrap_or(0.0).max(0.0), v.get("to").and_then(Value::as_f64).unwrap_or(info.duration).min(info.duration));
            let h = v.get("height").and_then(Value::as_u64).map(|h| (h as u32).min(info.height)).unwrap_or(info.height);
            let (w, h) = (((info.width as f64 * h as f64 / info.height as f64 / 2.0).round() as u32 * 2).max(2), (h / 2 * 2).max(2));
            let fps = info.fps;
            let kbps = v.get("kbps").and_then(Value::as_u64).map(|k| k as u32).unwrap_or(((w as f64 * h as f64 * fps * if h < info.height { 0.08 } else { 0.1 }) / 1000.0).clamp(300.0, 20000.0) as u32);
            let mut enc = be.encoder(std::path::Path::new(out), &raea_io_media::EncodeOpts { width: w, height: h, fps, kbps, quality: None, nv12: true, realtime: false })?;
            let n = ((to - from) * fps).round().max(0.0) as usize;
            for i in 0..n {
                let fr = dec.frame_nv12_at(from + (i as f64 + 0.5) / fps - 1e-6)?;
                let pic = Picture { width: fr.width, height: fr.height, nv12: fr.rgba };
                let pic = if pic.width != w || pic.height != h { scale_nv12(&pic, w, h) } else { pic };
                enc.push_nv12(&pic.nv12, w as usize, h as usize)?;
            }
            // the stretch's sound at the mix rate, in the source's channels (mono stays mono, as ffmpeg keeps it)
            let sound = raea_io_media::backend("soft").and_then(|b| b.decode_audio(std::path::Path::new(path))).ok().filter(|a| a.len() > 0).map(|a| {
                let a = raea_io_media::soft_resample(&a, raea_io_media::audio::RATE);
                let (lo, hi) = (((from * a.rate as f64) as usize).min(a.len()), (((from + n as f64 / fps) * a.rate as f64) as usize).min(a.len()));
                let chans: Vec<Vec<f32>> = (0..a.channels.len().clamp(1, 2)).map(|c| a.channels[c.min(a.channels.len() - 1)][lo..hi].to_vec()).collect();
                raea_io_media::Audio { rate: a.rate, channels: chans }
            });
            let stats = enc.finish(sound.as_ref())?;
            Ok(json!({"frames": n, "width": w, "height": h, "fps": fps, "kbps": kbps, "bytes": stats.bytes}))
        }
        "frame.info" => {
            let p = st.pictures.get(&u(v, "frame")?).ok_or("that frame is gone")?;
            Ok(json!({"width": p.width, "height": p.height}))
        }
        "frame.png" => {
            let p = st.pictures.get(&u(v, "frame")?).ok_or("that frame is gone")?;
            let rgba = raea_io_media::yuv::nv12_to_rgba(&p.nv12, p.width as usize, p.height as usize);
            image::save_buffer(s(v, "path")?, &rgba, p.width, p.height, image::ExtendedColorType::Rgba8).map_err(|e| e.to_string())?;
            Ok(Value::Null)
        }
        "frame.scale" => {
            let p = st.pictures.get(&u(v, "frame")?).ok_or("that frame is gone")?;
            let (w, h) = ((u(v, "width")? / 2 * 2).max(2) as u32, (u(v, "height")? / 2 * 2).max(2) as u32);
            let q = scale_nv12(p, w, h);
            st.next += 1;
            let id = st.next;
            st.pictures.insert(id, q);
            Ok(json!({"frame": id, "width": w, "height": h}))
        }
        "frame.transform" => {
            let p = st.pictures.get(&u(v, "frame")?).ok_or("that frame is gone")?;
            let (w, h) = ((u(v, "width")? / 2 * 2).max(2) as u32, (u(v, "height")? / 2 * 2).max(2) as u32);
            let crop = v.get("crop").filter(|c| c.is_object()).map(|c| (c["left"].as_f64().unwrap_or(0.0), c["top"].as_f64().unwrap_or(0.0), c["width"].as_f64().unwrap_or(1.0), c["height"].as_f64().unwrap_or(1.0)));
            let q = transform_nv12(p, v.get("rotation").and_then(Value::as_u64).unwrap_or(0) as u32, v.get("flip").and_then(Value::as_bool).unwrap_or(false), crop, w, h, v.get("fit").and_then(Value::as_str).unwrap_or("fill"));
            st.next += 1;
            let id = st.next;
            st.pictures.insert(id, q);
            Ok(json!({"frame": id, "width": w, "height": h}))
        }
        "frame.free" => {
            st.pictures.remove(&u(v, "frame")?);
            Ok(Value::Null)
        }
        "adec.new" => {
            let desc = b64(v, "description")?;
            let d = raea_io_media::soft::packets::AudioPacketDecoder::new(s(v, "codec")?, Some(&desc), u(v, "rate")? as u32, u(v, "channels")? as u32)?;
            st.next += 1;
            let id = st.next;
            st.adecs.insert(id, d);
            Ok(json!({"id": id}))
        }
        "adec.decode" => {
            let (rate, planes) = st.adecs.get_mut(&u(v, "id")?).ok_or("no such decoder")?.decode(&b64(v, "data")?)?;
            let mut bytes = Vec::with_capacity(planes.iter().map(Vec::len).sum::<usize>() * 4);
            for p in &planes {
                for x in p {
                    bytes.extend_from_slice(&x.to_le_bytes());
                }
            }
            Ok(json!({"rate": rate, "channels": planes.len(), "frames": planes.first().map(Vec::len).unwrap_or(0), "data": enc64(&bytes)}))
        }
        "venc.codecs" => {
            #[cfg(target_vendor = "apple")]
            let hw = raea_io_media::apple::packets::hardware_encoders();
            #[cfg(not(target_vendor = "apple"))]
            let hw: Vec<String> = Vec::new();
            // what the engine's encoder writes today (avc), and what the hardware could (av01, vp09 only where present)
            Ok(json!({"hardware": hw, "avc": hw.iter().any(|c| c == "avc1"), "hevc": hw.iter().any(|c| c == "hvc1"), "av1": hw.iter().any(|c| c == "av01"), "vp9": hw.iter().any(|c| c == "vp09")}))
        }
        #[cfg(target_vendor = "apple")]
        "aenc.new" => {
            let e = raea_io_media::apple::aenc::AacEncoder::new(u(v, "rate")? as u32, u(v, "channels")? as u32, v.get("kbps").and_then(Value::as_u64).unwrap_or(128) as u32)?;
            let desc = enc64(&e.description());
            st.next += 1;
            let id = st.next;
            st.aencs.insert(id, e);
            Ok(json!({"id": id, "description": desc, "priming": raea_io_media::apple::aenc::AAC_PRIMING}))
        }
        #[cfg(target_vendor = "apple")]
        "aenc.encode" | "aenc.finish" => {
            let id = u(v, "id")?;
            let e = st.aencs.get_mut(&id).ok_or("no such encoder")?;
            let packets = if name == "aenc.finish" {
                e.finish()?
            } else {
                let bytes = b64(v, "data")?;
                let ch = u(v, "channels")? as usize;
                let floats: Vec<f32> = bytes.chunks_exact(4).map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]])).collect();
                let n = floats.len() / ch.max(1);
                let planes: Vec<&[f32]> = (0..ch).map(|c| &floats[c * n..(c + 1) * n]).collect();
                e.encode(&planes)?
            };
            Ok(json!({"packets": packets.iter().map(|p| json!({"data": enc64(&p.data), "pts": p.pts, "dur": p.dur})).collect::<Vec<_>>()}))
        }
        #[cfg(target_vendor = "apple")]
        "aenc.free" => {
            st.aencs.remove(&u(v, "id")?);
            Ok(Value::Null)
        }
        "adec.free" => {
            st.adecs.remove(&u(v, "id")?);
            Ok(Value::Null)
        }
        o => Err(format!("no native call {o}")),
    }
}
