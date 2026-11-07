use super::*;

/// Flashing past WCAG 2.3.1: more than three flashes in any one second, over enough of the frame to matter. A flash is a
/// pair of opposing changes in the same pixels, so a shape sliding past isn't one. Each window, a third of the frame wide
/// and high, stands in for WCAG's quarter of a 10-degree field (the real area depends on the screen and how far away it
/// is, which a file doesn't say), and a quarter of its pixels must swing together: by 0.10 relative luminance with the
/// darker end under 0.80, or by 20 of saturated red ((R-G-B)*320, where R/(R+G+B) is at least 0.8). Every decoded frame
/// counts, on its own timestamp, and the picture is sampled twice, by area and by point, so fine patterns don't average
/// away. A screen for review, not a Harding or PEAT certification; an interlaced picture isn't split into its fields.
const FLASH_W: usize = 192;
const FLASH_MOST: usize = 6;
const FLASH_PAIR_S: f64 = 1.0 + 1e-9;

/// A pixel in one measure: its value, and whether it can count. Red's value runs on through the saturation line, and a change
/// counts when either end of it is saturated; a one-level step across the line isn't a jump from zero.
#[derive(Clone, Copy)]
struct Px {
    value: f32,
    counts: bool,
}

struct Swing {
    at: f64,
    direction: f32,
    moved: Vec<bool>,
    plain: bool,
    kept: bool,
}

/// One window's swings in one measure: the ones an opposite swing within a second moves back in the same pixels. The
/// general measure leaves out swings where another quarter of the window moved the other way at once (a moving pattern).
struct Swings {
    by: f32,
    under: f32,
    mixed_counts: bool,
    pivot: Vec<Px>,
    extreme: Vec<Px>,
    at: f64,
    direction: f32,
    recent: std::collections::VecDeque<Swing>,
    ends: Vec<f64>,
}

impl Swings {
    fn new(by: f32, under: f32, mixed_counts: bool) -> Swings {
        Swings { by, under, mixed_counts, pivot: Vec::new(), extreme: Vec::new(), at: 0.0, direction: 0.0, recent: Default::default(), ends: Vec::new() }
    }

    fn moves(&self, a: Px, b: Px, direction: f32) -> bool {
        (b.value - a.value) * direction >= self.by && a.value.min(b.value) < self.under && (a.counts || b.counts)
    }

    fn changed(&self, earlier: &[Px], later: &[Px], direction: f32) -> bool {
        earlier.iter().zip(later).filter(|(a, b)| self.moves(**a, **b, direction)).count() * 4 >= earlier.len()
    }

    fn push(&mut self, t: f64, px: &[Px]) {
        if self.pivot.is_empty() {
            self.pivot = px.to_vec();
            self.extreme = px.to_vec();
            self.at = t;
            return;
        }
        if self.direction == 0.0 {
            for direction in [1.0, -1.0] {
                if self.changed(&self.pivot, px, direction) {
                    self.direction = direction;
                    self.extreme = px.to_vec();
                    self.at = t;
                    break;
                }
            }
            return;
        }
        if self.changed(&self.extreme, px, -self.direction) {
            self.record();
            self.pivot = std::mem::replace(&mut self.extreme, px.to_vec());
            self.at = t;
            self.direction = -self.direction;
        } else if self.changed(&self.pivot, px, self.direction) && self.went_further(px) {
            self.extreme = px.to_vec();
        }
    }

    /// Whether the pixels swinging from the pivot went further than the saved extreme, judged on those pixels alone: the
    /// rest of the window can dim by less than counts and still outweigh a patch that's flashing.
    fn went_further(&self, px: &[Px]) -> bool {
        let further: f32 = self.pivot.iter().zip(px).zip(&self.extreme).filter(|((p, n), _)| self.moves(**p, **n, self.direction)).map(|((_, n), e)| (n.value - e.value) * self.direction).sum();
        further > 0.0
    }

    fn record(&mut self) {
        if !self.changed(&self.pivot, &self.extreme, self.direction) {
            return;
        }
        let moved: Vec<bool> = self.pivot.iter().zip(&self.extreme).map(|(a, b)| self.moves(*a, *b, self.direction)).collect();
        let plain = self.mixed_counts || !self.changed(&self.pivot, &self.extreme, -self.direction);
        let (at, direction) = (self.at, self.direction);
        let mut kept = false;
        for other in self.recent.iter_mut().rev() {
            if at - other.at > FLASH_PAIR_S {
                break;
            }
            let same = moved.iter().zip(&other.moved).filter(|(x, y)| **x && **y).count();
            if other.direction == -direction && same * 4 >= moved.len() {
                other.kept = true;
                kept = true;
            }
        }
        self.recent.push_back(Swing { at, direction, moved, plain, kept });
        while self.recent.front().is_some_and(|s| at - s.at > FLASH_PAIR_S) {
            self.retire();
        }
    }

    /// The oldest swing leaves the second it could still pair within; only the seconds' ends are kept for good.
    fn retire(&mut self) {
        if let Some(s) = self.recent.pop_front()
            && s.plain
            && s.kept
        {
            self.ends.push(s.at);
        }
    }

    /// The most swings in any one second, and when that second starts.
    fn busiest(&mut self) -> (usize, f64) {
        if self.direction != 0.0 {
            self.record();
            self.direction = 0.0;
        }
        while !self.recent.is_empty() {
            self.retire();
        }
        self.ends.sort_by(f64::total_cmp);
        let (mut best, mut from, mut j) = (0, 0.0, 0);
        for i in 0..self.ends.len() {
            while self.ends[i] - self.ends[j] >= 1.0 - 1e-9 {
                j += 1;
            }
            if i - j + 1 > best {
                (best, from) = (i - j + 1, self.ends[j]);
            }
        }
        (best, from)
    }
}

/// Frames of two pictures side by side, each `w` by `h` RGB: the area sample and the point sample of one frame.
pub(crate) struct FlashScreen {
    w: usize,
    h: usize,
    last: Vec<u8>,
    linear: Vec<f32>,
    luma: Vec<Px>,
    red: Vec<Px>,
    cut: Vec<Px>,
    windows: Vec<(usize, usize, Swings, Swings)>,
}

impl FlashScreen {
    pub(crate) fn new(w: usize, h: usize) -> FlashScreen {
        let (ww, wh) = (w / 3, h / 3);
        let places = |size: usize, window: usize| {
            let mut at: Vec<usize> = (0..=size - window).step_by((size / 6).max(1)).collect();
            at.push(size - window);
            at.dedup();
            at
        };
        let mut windows = Vec::new();
        for sample in 0..2 {
            for &y in &places(h, wh) {
                for &x in &places(w, ww) {
                    windows.push((x + sample * w, y, Swings::new(0.10, 0.80, false), Swings::new(20.0, f32::INFINITY, true)));
                }
            }
        }
        let linear = (0..=255u32)
            .map(|c| {
                let c = c as f32 / 255.0;
                if c <= 0.04045 { c / 12.92 } else { ((c + 0.055) / 1.055).powf(2.4) }
            })
            .collect();
        FlashScreen { w, h, last: Vec::new(), linear, luma: Vec::new(), red: Vec::new(), cut: Vec::new(), windows }
    }

    pub(crate) fn push(&mut self, rgb: &[u8], t: f64) {
        if rgb == self.last {
            return;
        }
        self.luma.clear();
        self.red.clear();
        for px in rgb.chunks_exact(3) {
            let (r, g, b) = (self.linear[px[0] as usize], self.linear[px[1] as usize], self.linear[px[2] as usize]);
            self.luma.push(Px { value: 0.2126 * r + 0.7152 * g + 0.0722 * b, counts: true });
            let sum = r + g + b + 1e-9;
            self.red.push(Px { value: ((r - g - b) * 320.0).max(0.0), counts: r / sum >= 0.8 });
        }
        let (stride, ww, wh) = (self.w * 2, self.w / 3, self.h / 3);
        for (x, y, general, saturated) in &mut self.windows {
            self.cut.clear();
            for r in 0..wh {
                self.cut.extend_from_slice(&self.luma[(*y + r) * stride + *x..][..ww]);
            }
            general.push(t, &self.cut);
            self.cut.clear();
            for r in 0..wh {
                self.cut.extend_from_slice(&self.red[(*y + r) * stride + *x..][..ww]);
            }
            saturated.push(t, &self.cut);
        }
        self.last.clear();
        self.last.extend_from_slice(rgb);
    }

    /// (swings in the worst second, when it starts, which measure) when past the limit
    pub(crate) fn finish(mut self) -> Option<(usize, f64, &'static str)> {
        let mut worst: Option<(usize, f64, &'static str)> = None;
        for (_, _, general, saturated) in &mut self.windows {
            for (swings, kind) in [(general, "general"), (saturated, "red")] {
                let (n, from) = swings.busiest();
                if n > FLASH_MOST && worst.is_none_or(|w| n > w.0) {
                    worst = Some((n, from, kind));
                }
            }
        }
        worst
    }
}

/// Every decoded frame, once: -fps_mode from FFmpeg 5.1 on, -vsync before it.
fn passthrough() -> [&'static str; 2] {
    let out = Command::new("ffmpeg").args(["-hide_banner", "-version"]).output().map(|o| o.stdout).unwrap_or_default();
    let head = String::from_utf8_lossy(&out);
    let version = head.split("version ").nth(1).and_then(|v| {
        let mut parts = v.trim_start_matches('n').split(|c: char| !c.is_ascii_digit());
        Some((parts.next()?.parse::<u32>().ok()?, parts.next()?.parse::<u32>().ok()?))
    });
    [if version.is_some_and(|v| v < (5, 1)) { "-vsync" } else { "-fps_mode" }, "passthrough"]
}

/// A frame's number and time from showinfo's log, exact: its `pts` in ticks of the time base. `pts_time` is rounded to six
/// significant digits, which is enough to push a legal third flash across the one-second line.
pub(crate) fn showinfo_frame(line: &str, base: Option<(i64, i64)>) -> Option<(usize, f64)> {
    let rest = line.split(" n:").nth(1)?;
    let n = rest.split_whitespace().next()?.parse().ok()?;
    let pts: i64 = rest.split("pts:").nth(1)?.split_whitespace().next()?.parse().ok()?;
    let (num, den) = base?;
    Some((n, pts as f64 * num as f64 / den as f64))
}

pub(crate) fn showinfo_base(line: &str) -> Option<(i64, i64)> {
    let (num, den) = line.split("config in time_base:").nth(1)?.split(',').next()?.trim().split_once('/')?;
    Some((num.parse().ok()?, den.parse().ok()?))
}

/// A line's severity under `-loglevel level+`: the first bracket after any `[context @ address]`. Searching the whole line
/// would take a file's own metadata for an error when its title happens to say one.
pub(crate) fn log_level(line: &str) -> Option<&str> {
    let mut rest = line.trim_start();
    if rest.starts_with('[') && rest[..rest.find(']')?].contains(" @ ") {
        rest = rest[rest.find(']')? + 1..].trim_start();
    }
    let rest = rest.strip_prefix('[')?;
    Some(&rest[..rest.find(']')?])
}

pub(crate) fn flashing(video: &str, probe: &Probe, issues: &mut Vec<Issue>, passed: &mut Vec<String>) -> R<()> {
    let w = FLASH_W;
    let h = (((w as f64 * probe.h / probe.w.max(1.0)) / 2.0).round() as usize * 2).max(6);
    let graph = format!("showinfo,split=2[area][point];[area]scale={w}:{h}:flags=area[a];[point]scale={w}:{h}:flags=neighbor[b];[a][b]hstack=inputs=2,format=rgb24");
    let sync = passthrough();
    let mut child = Command::new("ffmpeg")
        .args(["-hide_banner", "-nostats", "-nostdin", "-loglevel", "level+info", "-i", video, "-map", "0:v:0", "-vf", &graph, sync[0], sync[1], "-f", "rawvideo", "-"])
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| format!("ffmpeg: {e}"))?;
    let mut out = child.stdout.take().unwrap();
    let err = child.stderr.take().unwrap();
    let (times, clock) = std::sync::mpsc::channel();
    // read apart from the frames, so that neither pipe can fill while the other is being waited on
    let log = std::thread::spawn(move || {
        let mut err = std::io::BufReader::new(err);
        let (mut line, mut base, mut errors, mut first) = (Vec::new(), None, 0usize, String::new());
        while std::io::BufRead::read_until(&mut err, b'\n', &mut line).is_ok_and(|n| n > 0) {
            let text = String::from_utf8_lossy(&line);
            if matches!(log_level(&text), Some("error" | "fatal" | "panic")) {
                errors += 1;
                if first.is_empty() {
                    first = text.trim().to_string();
                }
            } else if text.trim_start().starts_with("[Parsed_showinfo") {
                base = showinfo_base(&text).or(base);
                if let Some(frame) = showinfo_frame(&text, base) {
                    let _ = times.send(frame);
                }
            }
            line.clear();
        }
        (errors, first)
    });
    let mut screen = FlashScreen::new(w, h);
    let mut buf = vec![0u8; w * 2 * h * 3];
    let mut frames = 0;
    let mut lost = false;
    while std::io::Read::read_exact(&mut out, &mut buf).is_ok() {
        // the frame's own line is written before its pixels, so one that doesn't come isn't going to
        match clock.recv_timeout(std::time::Duration::from_secs(60)) {
            Ok((n, t)) if n == frames => screen.push(&buf, t),
            _ => {
                lost = true;
                let _ = child.kill();
                break;
            }
        }
        frames += 1;
    }
    drop(out);
    let status = child.wait().map_err(|e| format!("ffmpeg: {e}"))?;
    let (errors, first) = log.join().unwrap_or_default();
    if lost {
        return Err(format!("{video}: flashing can't be screened, frame {frames} has no timestamp of its own"));
    }
    if !status.success() || frames == 0 {
        return Err(format!("{video}: flashing can't be screened, FFmpeg decoded {frames} frames ({first})"));
    }
    if errors > 0 {
        issues.push(Issue {
            rule: "decode",
            p: 1,
            at: None,
            says: format!("FFmpeg hit {errors} errors decoding the picture, so it may be damaged and flashing was only screened on what decoded: {first}"),
        });
    }
    match screen.finish() {
        Some((swings, at, kind)) => issues.push(Issue {
            rule: "flash",
            p: 1,
            at: Some(at),
            says: format!("{swings} opposing {kind} changes in one second, past WCAG 2.3.1's limit of 3 flashes (6 changes); flashing like this can cause seizures: slow it, shrink it or lower its contrast"),
        }),
        None if errors == 0 => passed.push("flashing: no more than 3 flashes a second found (a screen, not a certification)".to_string()),
        None => {}
    }
    Ok(())
}
