//! `raea neural mlxdlss DIR -o OUT` (docs/raytracing.feature RT2): DLSS 5's neural rendering on a render, on the Mac,
//! through MLX-DLSS (github.com/iamwavecut/MLX-DLSS, Apache-2.0), fed raea's own motion vectors and depth (`render
//! --aov`). An optional integration: the mlxdlss binary and the weights are the founder's (the weights are NVIDIA's,
//! extracted from their own DLSS install, and never ship with raea).

use std::path::{Path, PathBuf};

use raea_runtime_bundle::{R, flag};

pub const USAGE: &str = "raea neural mlxdlss DIR -o OUT [--model M.dlssmodel] [--intensity X] [--profile P] [--skin-structure X] [--local-structure X] [--steady A]";

fn tool_and_model(args: &[String]) -> R<(PathBuf, PathBuf)> {
    let bin = std::env::var_os("RAEA_MLXDLSS").map(PathBuf::from).filter(|p| p.is_file()).ok_or(
        "raea neural mlxdlss needs your own MLX-DLSS build: set RAEA_MLXDLSS to its mlxdlss binary (github.com/iamwavecut/MLX-DLSS, `swift build -c release`)",
    )?;
    let model = flag(args, "--model").or_else(|| std::env::var("RAEA_DLSS_MODEL").ok()).map(PathBuf::from).filter(|p| p.exists()).ok_or(
        "raea neural mlxdlss needs weights you extracted yourself (MLX-DLSS's `mlxdlss-weights extract` from your NVIDIA DLSS install): --model or RAEA_DLSS_MODEL names the .dlssmodel; raea doesn't ship them",
    )?;
    Ok((bin, model))
}

/// The frames of a `render --aov` folder, in order: (picture, motion, depth).
fn frames(dir: &Path) -> R<Vec<(PathBuf, PathBuf, PathBuf)>> {
    let mut pngs: Vec<PathBuf> = std::fs::read_dir(dir).map_err(|e| format!("{}: {e}", dir.display()))?.flatten().map(|e| e.path()).filter(|p| p.extension().is_some_and(|x| x == "png") && p.file_name().is_some_and(|n| n.to_string_lossy().starts_with("frame-"))).collect();
    pngs.sort();
    if pngs.is_empty() {
        return Err(format!("{}: no frame-NNNNN.png (make it with raea render EDIT.tsx -o {} --aov)", dir.display(), dir.display()));
    }
    pngs.into_iter()
        .map(|p| {
            let stem = p.file_stem().unwrap().to_string_lossy().into_owned();
            let (m, d) = (dir.join("motion").join(format!("{stem}.f32")), dir.join("depth").join(format!("{stem}.f32")));
            if !m.is_file() || !d.is_file() {
                return Err(format!("{}: no motion or depth beside it (render with --aov)", p.display()));
            }
            Ok((p, m, d))
        })
        .collect()
}

/// A 0..1 channel as an 8-bit one (rounded, clamped).
fn to_byte(x: f32) -> u8 {
    (x.clamp(0.0, 1.0) * 255.0).round() as u8
}

fn f32s(p: &Path) -> R<Vec<f32>> {
    let b = std::fs::read(p).map_err(|e| format!("{}: {e}", p.display()))?;
    Ok(b.chunks_exact(4).map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]])).collect())
}

fn write_f32s(p: &Path, v: impl Iterator<Item = f32>) -> R<()> {
    std::fs::write(p, v.flat_map(f32::to_le_bytes).collect::<Vec<u8>>()).map_err(|e| format!("{}: {e}", p.display()))
}

pub fn neural(args: &[String]) -> R<String> {
    if args.first().map(String::as_str) != Some("mlxdlss") || args.len() < 2 {
        return Err(format!("usage: {USAGE}"));
    }
    let dir = PathBuf::from(&args[1]);
    let out = PathBuf::from(flag(args, "-o").ok_or(format!("usage: {USAGE}"))?);
    let (bin, model) = tool_and_model(args)?;
    let list = frames(&dir)?;
    let (w, h) = image::image_dimensions(&list[0].0).map_err(|e| format!("{}: {e}", list[0].0.display()))?;
    // the inputs as run-sequence reads them: float32 NHWC, rows top to bottom. Colour is the frame's sRGB values in
    // 0..1 (as MLX-DLSS's own video path hands it frames); motion is negated, since run-sequence's normalized-uv adds
    // it to a pixel to find where it was, and Bevy's is where it is minus where it was; depth is Bevy's reverse Z
    let work = std::env::temp_dir().join(format!("raea-neural-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&work);
    std::fs::create_dir_all(&work).map_err(|e| e.to_string())?;
    // one run-sequence a shot: the network keeps a history, and carried over a cut it painted the last shot's face
    // onto the next one's
    let starts = cuts(&list, w as usize, h as usize)?;
    // each shot run, steadied and written before the next, and its float files deleted: a whole render's at once
    // (colour, motion and the network's output, 70 MB a 1080p frame) filled the disk
    std::fs::create_dir_all(&out).map_err(|e| format!("{}: {e}", out.display()))?;
    let a: f32 = flag(args, "--steady").map(|v| v.parse::<f32>().ok().filter(|a| *a > 0.0 && *a <= 1.0).ok_or("--steady takes 0..1 (1: off)")).transpose()?.unwrap_or(0.25);
    let mut steady = Steady { w: w as usize, h: h as usize, a, hist: None };
    for (k, &s0) in starts.iter().enumerate() {
        let s1 = starts.get(k + 1).copied().unwrap_or(list.len());
        let mut cmd = std::process::Command::new(&bin);
        cmd.arg("run-sequence").arg(&model);
        for (i, (png, m, d)) in list.iter().enumerate().take(s1).skip(s0) {
            let rgb = image::open(png).map_err(|e| format!("{}: {e}", png.display()))?.to_rgb8();
            if rgb.dimensions() != (w, h) {
                return Err(format!("{}: {}x{}, the first frame is {w}x{h}", png.display(), rgb.width(), rgb.height()));
            }
            let (c, mo) = (work.join(format!("c{i:05}.f32")), work.join(format!("m{i:05}.f32")));
            write_f32s(&c, rgb.as_raw().iter().map(|&b| b as f32 / 255.0))?;
            // a shot's first frame has no last frame to have moved from
            let mv = if i == s0 { vec![0.0; (w * h * 2) as usize] } else { f32s(m)? };
            if mv.len() != (w * h * 2) as usize {
                return Err(format!("{}: {} floats, a {w}x{h} frame's motion is {}", m.display(), mv.len(), w * h * 2));
            }
            write_f32s(&mo, mv.into_iter().map(|v| -v))?;
            cmd.arg("--input").arg(&c).arg("--motion").arg(&mo).arg("--depth").arg(d);
        }
        let shot = work.join(format!("shot{k}"));
        cmd.args(["--input-format", "rgb-temporal-reference", "--motion-format", "normalized-uv", "--depth-inverted", "true"]);
        cmd.args(["--width", &w.to_string(), "--height", &h.to_string()]).arg("--output-dir").arg(&shot);
        for key in ["--intensity", "--profile", "--skin-structure", "--local-structure"] {
            if let Some(v) = flag(args, key) {
                cmd.args([key, &v]);
            }
        }
        let st = cmd.stdout(std::process::Stdio::null()).status().map_err(|e| format!("{}: {e}", bin.display()))?;
        if !st.success() {
            return Err(format!("mlxdlss run-sequence failed on frames {s0}..{s1} ({st})"));
        }
        for i in s0..s1 {
            let _ = std::fs::remove_file(work.join(format!("c{i:05}.f32")));
            let _ = std::fs::remove_file(work.join(format!("m{i:05}.f32")));
        }
        steady.hist = None;
        for (i, (png, m, d)) in list.iter().enumerate().take(s1).skip(s0) {
            let f = shot.join(format!("{:06}.f32", i - s0));
            let v = f32s(&f)?;
            if v.len() != (w * h * 3) as usize {
                return Err(format!("mlxdlss frame {i}: {} floats, not {w}x{h}x3", v.len()));
            }
            let raw: Vec<f32> = image::open(png).map_err(|e| format!("{}: {e}", png.display()))?.to_rgb8().as_raw().iter().map(|&b| b as f32 / 255.0).collect();
            let v = steady.frame(&raw, &v, &f32s(m)?, &f32s(d)?);
            let px: Vec<u8> = v.iter().copied().map(to_byte).collect();
            let dest = out.join(png.file_name().unwrap());
            image::save_buffer(&dest, &px, w, h, image::ColorType::Rgb8).map_err(|e| format!("{}: {e}", dest.display()))?;
        }
        let _ = std::fs::remove_dir_all(&shot);
    }
    eprintln!("raea: {} shot(s), cut before frames {:?}", starts.len(), &starts[1..]);
    let _ = std::fs::remove_dir_all(&work);
    Ok(format!("APPLIED  {} frames through DLSS 5 neural rendering (MLX-DLSS, raea's motion and depth) to {}", list.len(), out.display()))
}

/// `--steady A`: the network re-invents fine detail a little differently each frame (skin grain, a silhouette's edge),
/// which reads as shimmer where raea's own frames are still. Its change to each frame (its output minus raea's) is
/// carried along raea's motion vectors and blended in at A a frame (an exponential history, as TAA's), then added back
/// onto raea's frame: the look stays, the shimmer goes. A cut (`cuts`) and a pixel that came
/// out from behind something (its depth where it was isn't its depth now) start again from this frame's change.
/// 1 turns it off. On the faces reel's still shot: shimmer 0.21 -> about a quarter of it (RT2).
struct Steady {
    w: usize,
    h: usize,
    a: f32,
    /// the last frame: its carried change (3 a pixel), raea's frame and depth (None at a shot's start)
    hist: Option<(Vec<f32>, Vec<f32>, Vec<f32>)>,
}

impl Steady {
    fn frame(&mut self, raw: &[f32], net: &[f32], motion: &[f32], depth: &[f32]) -> Vec<f32> {
        let (w, h, n) = (self.w, self.h, self.w * self.h);
        let change: Vec<f32> = net.iter().zip(raw).map(|(o, r)| o - r).collect();
        let carried = if self.hist.is_none() || self.a >= 1.0 {
            change
        } else {
            let (hc, _, hd) = self.hist.as_ref().unwrap();
            let mut c = vec![0f32; n * 3];
            for y in 0..h {
                for x in 0..w {
                    let i = y * w + x;
                    // where this pixel was: Bevy's motion is this frame's UV minus the last's
                    let (px, py) = (x as f32 - motion[i * 2] * w as f32, y as f32 - motion[i * 2 + 1] * h as f32);
                    let (fx, fy) = (px.floor(), py.floor());
                    let inside = fx >= 0.0 && fy >= 0.0 && fx < (w - 1) as f32 && fy < (h - 1) as f32;
                    let (x0, y0) = (fx as usize, fy as usize);
                    let was = if inside { hd[y0 * w + x0].max(hd[y0 * w + x0 + 1]).max(hd[(y0 + 1) * w + x0]).max(hd[(y0 + 1) * w + x0 + 1]) } else { -1.0 };
                    // reverse Z: close enough when within 2% (0 is the background, where any depth is the same)
                    let same = inside && ((was - depth[i]).abs() <= 0.02 * depth[i].max(was) || (was == 0.0 && depth[i] == 0.0));
                    for k in 0..3 {
                        let now = change[i * 3 + k];
                        c[i * 3 + k] = if same {
                            let (tx, ty) = (px - fx, py - fy);
                            let s = |xx: usize, yy: usize| hc[(yy * w + xx) * 3 + k];
                            let old = (s(x0, y0) * (1.0 - tx) + s(x0 + 1, y0) * tx) * (1.0 - ty) + (s(x0, y0 + 1) * (1.0 - tx) + s(x0 + 1, y0 + 1) * tx) * ty;
                            old + (now - old) * self.a
                        } else {
                            now
                        };
                    }
                }
            }
            c
        };
        let out = raw.iter().zip(&carried).map(|(r, c)| r + c).collect();
        self.hist = Some((carried, raw.to_vec(), depth.to_vec()));
        out
    }
}

/// Where the shots start (0 first): a frame is a cut when most of what it shows wasn't where its motion vectors say it
/// came from, by depth (reverse Z within 2%), or its foreground grew or shrank by half. raea's motion is the scene's,
/// so a cut between two people facing the same way (their silhouettes nearly the same) is still found.
fn cuts(list: &[(PathBuf, PathBuf, PathBuf)], w: usize, h: usize) -> R<Vec<usize>> {
    let mut starts = vec![0];
    let mut last = f32s(&list[0].2)?;
    for (i, (_, m, d)) in list.iter().enumerate().skip(1) {
        let (mv, depth) = (f32s(m)?, f32s(d)?);
        if mv.len() != w * h * 2 || depth.len() != w * h {
            return Err(format!("{}: not a {w}x{h} frame's motion and depth", m.display()));
        }
        let (mut fg, mut lost, mut was_fg) = (0usize, 0usize, 0usize);
        for y in (0..h).step_by(2) {
            for x in (0..w).step_by(2) {
                let k = y * w + x;
                was_fg += (last[k] > 0.0) as usize;
                if depth[k] <= 0.0 {
                    continue;
                }
                fg += 1;
                let (px, py) = ((x as f32 - mv[k * 2] * w as f32).round(), (y as f32 - mv[k * 2 + 1] * h as f32).round());
                let ok = px >= 0.0 && py >= 0.0 && (px as usize) < w && (py as usize) < h && {
                    let was = last[py as usize * w + px as usize];
                    (was - depth[k]).abs() <= 0.02 * depth[k].max(was)
                };
                lost += (!ok) as usize;
            }
        }
        let grew = fg.max(was_fg) as f32 / fg.min(was_fg).max(1) as f32;
        if (fg > 0 && lost as f32 > 0.3 * fg as f32) || grew > 1.5 {
            starts.push(i);
        }
        last = depth;
    }
    Ok(starts)
}
