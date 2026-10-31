use super::*;

/// Encoder blotching in dark areas (found 10-24: VideoToolbox smears block-shaped patches over near-black where the
/// picture is flat). Per 8x8 block of luma, at one frame every two seconds: among dark blocks that are flat inside
/// (a range under `FLAT` levels), the share whose mean steps from a flat dark neighbour's by more than `STEP` and less
/// than `MOST` levels (bigger steps are the picture's own edges). Of those, only a patch's edges count: not steps that
/// run straight on for `RUN` blocks (the picture's own faint edges: a lossless night scene of dark panels scored 0.053
/// before, more than VideoToolbox's blotches), and only steps the level comes back from within `BACK` blocks (a glow's
/// 8-bit rings keep stepping one way: a lossless one scored 0.043 without). Calibrated 10-25 on 16 frames each: three
/// VideoToolbox exports (the tutorial, two night-market films) scored 0.012-0.014 (median), every x264 or lossless
/// encode at most 0.007.
pub(crate) const FLAT: u8 = 2;
pub(crate) const STEP: f32 = 0.5;
pub(crate) const MOST: f32 = 4.0;
pub(crate) const RUN: usize = 4;
pub(crate) const BACK: usize = 4;
pub(crate) const BLOTCHY: f64 = 0.009;

pub(crate) fn blotch_score(y: &[u8], w: usize, h: usize) -> Option<f64> {
    let (bw, bh) = (w / 8, h / 8);
    let mut mean = vec![0f32; bw * bh];
    let mut dark = vec![false; bw * bh];
    for by in 0..bh {
        for bx in 0..bw {
            let (mut sum, mut lo, mut hi) = (0u32, 255u8, 0u8);
            for r in 0..8 {
                let row = &y[(by * 8 + r) * w + bx * 8..][..8];
                for &v in row {
                    sum += v as u32;
                    lo = lo.min(v);
                    hi = hi.max(v);
                }
            }
            let m = sum as f32 / 64.0;
            mean[by * bw + bx] = m;
            dark[by * bw + bx] = m < 40.0 && hi - lo < FLAT;
        }
    }
    if dark.iter().filter(|d| **d).count() < 400 {
        return None;
    }
    // each pair's step, signed (0 when it isn't one): across the boundary to the right (h) and the one below (v)
    let step = |i: usize, j: usize| {
        let d = mean[j] - mean[i];
        if dark[i] && dark[j] && d.abs() > STEP && d.abs() < MOST { d.signum() as i8 } else { 0 }
    };
    let mut h = vec![0i8; bw * bh];
    let mut v = vec![0i8; bw * bh];
    let mut pairs = 0u32;
    for by in 0..bh {
        for bx in 0..bw {
            let i = by * bw + bx;
            if bx + 1 < bw {
                pairs += (dark[i] && dark[i + 1]) as u32;
                h[i] = step(i, i + 1);
            }
            if by + 1 < bh {
                pairs += (dark[i] && dark[i + bw]) as u32;
                v[i] = step(i, i + bw);
            }
        }
    }
    // A step that runs on, the same way, for RUN blocks along its boundary is an edge or a gradient band the picture
    // draws (a flat panel a level off its ground); the encoder's patches are ragged. Only those count.
    let ragged = |s: &[i8], along: usize, len: usize, i: usize, at: usize| {
        let (mut back, mut on) = (0, 0);
        while back < RUN && back < at && s[i - (back + 1) * along] == s[i] {
            back += 1;
        }
        while on < RUN && at + on + 1 < len && s[i + (on + 1) * along] == s[i] {
            on += 1;
        }
        back + on + 1 < RUN
    };
    // and a patch closes: within `BACK` blocks across, the level steps back the other way. A gradient's bands (a glow
    // drawn in 8 bits shows rings) keep stepping the same way.
    let closes = |s: &[i8], across: usize, len: usize, i: usize, at: usize| {
        (1..=BACK).any(|k| (k <= at && s[i - k * across] == -s[i]) || (at + k < len && s[i + k * across] == -s[i]))
    };
    let mut steps = 0u32;
    for by in 0..bh {
        for bx in 0..bw {
            let i = by * bw + bx;
            steps += (h[i] != 0 && ragged(&h, bw, bh, i, by) && closes(&h, 1, bw - 1, i, bx)) as u32;
            steps += (v[i] != 0 && ragged(&v, 1, bw, i, bx) && closes(&v, bw, bh - 1, i, by)) as u32;
        }
    }
    (pairs > 0).then(|| steps as f64 / pairs as f64)
}

/// The crf an x264 encode was made at, from the settings x264 writes into its stream ("x264 - core ... crf=12.0").
pub(crate) fn x264_crf(head: &[u8]) -> Option<f64> {
    let at = head.windows(11).position(|w| w == b"x264 - core")?;
    let opts = &head[at..(at + 4096).min(head.len())];
    let c = opts.windows(4).position(|w| w == b"crf=")?;
    let n: String = opts[c + 4..].iter().take_while(|b| b.is_ascii_digit() || **b == b'.').map(|b| *b as char).collect();
    n.parse().ok()
}

/// x264 at this crf or under keeps near-blacks (every one measured 10-24 and 10-25 was clean to the eye); what steps
/// in its darks is the picture's own (8-bit glows and faint panels), which blotch_score can't always tell apart
pub(crate) const X264_KEEPS: f64 = 16.0;

pub(crate) fn blotching(video: &str, probe: &Probe, issues: &mut Vec<Issue>, passed: &mut Vec<String>) -> R<()> {
    let mut head = Vec::new();
    if let Ok(f) = std::fs::File::open(video) {
        let _ = std::io::Read::read_to_end(&mut std::io::Read::take(f, 8 << 20), &mut head);
    }
    if let Some(crf) = x264_crf(&head).filter(|c| *c <= X264_KEEPS) {
        passed.push(format!("blotching: encoded by x264 at crf {crf}, which keeps the darks"));
        return Ok(());
    }
    let (w, h) = (probe.w as usize / 8 * 8, probe.h as usize / 8 * 8);
    let mut child = Command::new("ffmpeg")
        .args(["-v", "error", "-i", video, "-vf", &format!("fps=0.5,crop={w}:{h}:0:0,format=gray"), "-f", "rawvideo", "-"])
        .stdout(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| format!("ffmpeg: {e}"))?;
    let mut out = child.stdout.take().unwrap();
    let mut buf = vec![0u8; w * h];
    let (mut bad, mut seen, mut worst) = (Vec::new(), 0, 0f64);
    let mut t = 0.0;
    while std::io::Read::read_exact(&mut out, &mut buf).is_ok() {
        if let Some(s) = blotch_score(&buf, w, h) {
            seen += 1;
            worst = worst.max(s);
            if std::env::var_os("RAEA_CHECK_DEBUG").is_some() {
                eprintln!("blotch {t:6.1}s {s:.3}");
            }
            if s > BLOTCHY {
                bad.push((t, s));
            }
        }
        t += 2.0;
    }
    let _ = child.wait();
    if !bad.is_empty() {
        let at = bad[0].0;
        let share = bad.len() as f64 / seen.max(1) as f64;
        issues.push(Issue {
            rule: "blotching",
            p: if share > 0.2 { 1 } else { 2 },
            at: Some(at),
            says: format!(
                "blocky patches in dark areas in {} of {seen} sampled frames (worst {:.0}% of dark blocks stepping); the encoder starved the near-blacks: export as PNG frames and encode with more bits for darks",
                bad.len(),
                worst * 100.0
            ),
        });
    } else if seen > 0 {
        passed.push(format!("blotching: dark areas clean in {seen} sampled frames"));
    }
    Ok(())
}
