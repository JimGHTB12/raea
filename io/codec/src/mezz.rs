//! Optimized media (lightspeed F4): a long-GOP source over 1080p gets an intra-only copy in the mezz folder, made in
//! the background, and every decoder opened on the source (play, scrub, render) reads the copy once it is complete. A
//! seek in it decodes one frame where the source's 1-2 s GOPs decoded ~25 (k4: p99 53 ms, the media engine's ~590 fps
//! at 4K). Until the copy is complete everything reads the source, so a render and a play at the same moment read the
//! same pixels. Apple's copies are ProRes 422 (VMAF 99.98 against k4's H.264) made by VideoToolbox; elsewhere they're
//! all-intra H.264 made by the soft backend (raea_io_media::mezz makes both).
//!
//! This is the index every platform shares: plain files. A copy is named by the source's fingerprint (its length and
//! first and last MiB), written to a `.partial` file and renamed when whole; the folder is kept under a cap by last use:
//! RAEA_MEZZ_CAP_GB (default 20) or the disk's free space less RAEA_MEZZ_RESERVE_GB (default 20), whichever is smaller,
//! and a copy that wouldn't fit isn't made. RAEA_MEZZ=0 reads sources only.

use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::SystemTime;

use crate::{Frame, Info, R, VideoDecoder};

/// A copy's extension: ProRes in QuickTime on Apple, H.264 in MP4 elsewhere.
pub const EXT: &str = if cfg!(target_vendor = "apple") { "mov" } else { "mp4" };

/// The cache folder: RAEA_MEZZ_DIR, else raea's caches' mezz/ (raea_io_home::caches: ~/Library/Caches/raea on Apple,
/// $XDG_CACHE_HOME/raea on Linux, %LOCALAPPDATA%\raea\cache on Windows).
pub fn dir() -> PathBuf {
    if let Some(d) = std::env::var_os("RAEA_MEZZ_DIR") {
        return PathBuf::from(d);
    }
    raea_io_home::caches().unwrap_or_else(|| std::env::temp_dir().join("raea")).join("mezz")
}

pub fn enabled() -> bool {
    std::env::var("RAEA_MEZZ").as_deref() != Ok("0")
}

/// The source's fingerprint: FNV-1a over its length and its first and last MiB, remembered per (path, length,
/// modified) so an open reads the file's ends once a process.
pub fn fingerprint(src: &Path) -> Option<String> {
    static SEEN: Mutex<Option<HashMap<(PathBuf, u64, SystemTime), String>>> = Mutex::new(None);
    let meta = std::fs::metadata(src).ok()?;
    let key = (src.to_path_buf(), meta.len(), meta.modified().ok()?);
    if let Some(f) = SEEN.lock().ok()?.as_ref().and_then(|m| m.get(&key)) {
        return Some(f.clone());
    }
    const MIB: u64 = 1 << 20;
    let mut f = File::open(src).ok()?;
    let mut h: u64 = 0xcbf29ce484222325;
    let mut eat = |b: &[u8]| {
        for x in b {
            h ^= *x as u64;
            h = h.wrapping_mul(0x100000001b3);
        }
    };
    eat(&meta.len().to_le_bytes());
    let mut buf = vec![0u8; MIB.min(meta.len()) as usize];
    f.read_exact(&mut buf).ok()?;
    eat(&buf);
    if meta.len() > MIB {
        f.seek(SeekFrom::Start(meta.len().saturating_sub(MIB))).ok()?;
        buf.clear();
        f.read_to_end(&mut buf).ok()?;
        eat(&buf);
    }
    let fp = format!("{h:016x}");
    SEEN.lock().ok()?.get_or_insert_with(HashMap::new).insert(key, fp.clone());
    Some(fp)
}

/// Where the whole copy of the source with fingerprint `fp` is kept.
pub fn copy_of(fp: &str) -> PathBuf {
    dir().join(format!("{fp}.{EXT}"))
}

/// The path a decoder should open for `src`: its complete copy when there is one, else `src` itself.
pub fn resolve(src: &Path) -> PathBuf {
    if !enabled() {
        return src.to_path_buf();
    }
    match fingerprint(src).map(|fp| copy_of(&fp)) {
        Some(c) if c.is_file() => {
            // last use, for the cap's order
            let _ = File::options().append(true).open(&c).and_then(|f| f.set_modified(SystemTime::now()));
            c
        }
        _ => src.to_path_buf(),
    }
}

/// Whether `src` already has its complete copy.
pub fn has(src: &Path) -> bool {
    fingerprint(src).is_some_and(|fp| copy_of(&fp).is_file())
}

/// What a copy is decided from: the source's video track as its container says, nothing decoded.
#[derive(Clone, Debug)]
pub struct Probe {
    /// the sample entry's four-character code (avc1, hvc1, ...)
    pub codec: String,
    pub width: f64,
    pub height: f64,
    pub fps: f64,
    /// the track's minimum frame duration as (value, timescale)
    pub frame: (i64, i32),
    pub secs: f64,
    /// the most frames in a row that aren't keyframes, counted up to 9 (more says the same)
    pub longest_gop: usize,
}

impl Probe {
    /// the track's minimum frame duration in seconds
    pub fn frame_secs(&self) -> f64 {
        self.frame.0 as f64 / self.frame.1 as f64
    }
    /// H.264 or HEVC whose keyframes lie more than 8 frames apart, at any size: a seek in it decodes a chain of frames.
    /// Such a source under 1080p gets copies of the ranges an edit uses, not the whole file.
    pub fn long_gop(&self) -> bool {
        matches!(self.codec.as_str(), "avc1" | "avc3" | "hvc1" | "hev1") && self.longest_gop > 8
    }
    /// Worth a whole copy: long-GOP over 1920x1080.
    pub fn wants(&self) -> bool {
        self.width * self.height > 1920.0 * 1080.0 && self.long_gop()
    }
    /// A copy's likely size: ~2 bits a pixel at the source's size, rate and length, plus a tenth (ProRes 422; k4's 4K25
    /// ran 410 Mbps).
    pub fn likely_bytes(&self) -> u64 {
        (self.width * self.height * self.fps.max(1.0) * self.secs.max(0.0) * 2.0 / 8.0 * 1.1) as u64
    }
    /// The frames (counting the minimum frame duration) a copy of `start..end` seconds covers.
    pub fn frames(&self, start: f64, end: f64) -> (usize, usize) {
        let fd = self.frame_secs();
        let first = (start.max(0.0) / fd).floor() as usize;
        (first, ((end / fd).ceil() as usize).max(first + 1) - 1)
    }
}

/// A complete copy of `start..end` seconds of a source, where `first..=last` count the track's minimum frame duration
/// (a variable-rate phone clip's runs finer than its frames: 1/92 s at 30 fps): the copy's time t is the source's
/// start + t.
#[derive(Clone, Debug)]
pub struct RangeCopy {
    pub first: usize,
    pub last: usize,
    pub start: f64,
    pub end: f64,
    pub path: PathBuf,
}

/// The source's complete range copies, in order, named `FP@FIRST-LAST.EXT` beside the whole-file copies;
/// `frame_secs` gives the source's minimum frame duration, asked only when there are copies.
pub fn ranges(src: &Path, frame_secs: impl FnOnce() -> Option<f64>) -> Vec<RangeCopy> {
    if !enabled() {
        return Vec::new();
    }
    let Some(fp) = fingerprint(src) else { return Vec::new() };
    let prefix = format!("{fp}@");
    let Ok(rd) = std::fs::read_dir(dir()) else { return Vec::new() };
    let names: Vec<(PathBuf, usize, usize)> = rd
        .flatten()
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            let rest = name.strip_prefix(&prefix)?.strip_suffix(&format!(".{EXT}"))?;
            if rest.contains("partial") {
                return None;
            }
            let (a, b) = rest.split_once('-')?;
            Some((e.path(), a.parse().ok()?, b.parse().ok()?))
        })
        .collect();
    if names.is_empty() {
        return Vec::new();
    }
    let Some(fd) = frame_secs() else { return Vec::new() };
    let mut out: Vec<RangeCopy> = names.into_iter().map(|(path, first, last)| RangeCopy { first, last, start: first as f64 * fd, end: (last + 1) as f64 * fd, path }).collect();
    out.sort_by_key(|r| r.first);
    out
}

/// A copy being made: written to `part`, renamed to `out` by `finish`.
pub struct Job {
    pub out: PathBuf,
    pub part: PathBuf,
}

/// A copy of `src` named `name` (its fingerprint, `@FIRST-LAST` for a range) wanting about `need` bytes: room made
/// under the cap and the reserve left free on the disk after it, or why there isn't.
pub fn begin(src: &Path, name: &str, need: u64) -> R<Job> {
    std::fs::create_dir_all(dir()).map_err(|e| e.to_string())?;
    sweep();
    let (cap, free) = (cap_bytes(), free_bytes().unwrap_or(0));
    if free < need + reserve() || used_bytes() + need > cap {
        return Err(format!("{}: no room for a copy ({:.2} GB wanted, {:.1} GB free, cap {:.1} GB)", src.display(), need as f64 / 1e9, free as f64 / 1e9, cap as f64 / 1e9));
    }
    let part = dir().join(format!("{name}.{}.partial.{EXT}", std::process::id()));
    let _ = std::fs::remove_file(&part);
    Ok(Job { out: dir().join(format!("{name}.{EXT}")), part })
}

impl Job {
    /// The copy, whole: renamed into place and the folder pruned to its cap.
    pub fn finish(self) -> R<PathBuf> {
        std::fs::rename(&self.part, &self.out).map_err(|e| e.to_string())?;
        prune();
        Ok(self.out)
    }
}

/// Partial files left by a copy that was stopped (its process killed): removed once they're an hour old.
fn sweep() {
    let Ok(rd) = std::fs::read_dir(dir()) else { return };
    for e in rd.flatten() {
        let p = e.path();
        let old = e.metadata().and_then(|m| m.modified()).ok().and_then(|t| t.elapsed().ok()).is_some_and(|a| a.as_secs() > 3600);
        if p.to_string_lossy().contains(".partial.") && old {
            let _ = std::fs::remove_file(p);
        }
    }
}

/// What the disk keeps free whatever the copies want: RAEA_MEZZ_RESERVE_GB, default 20 GB.
fn reserve() -> u64 {
    (std::env::var("RAEA_MEZZ_RESERVE_GB").ok().and_then(|v| v.parse::<f64>().ok()).unwrap_or(20.0) * 1e9) as u64
}

/// The folder's cap: RAEA_MEZZ_CAP_GB (default 20 GB), and never more than the disk's free space less the reserve
/// counting what the copies already hold.
fn cap_bytes() -> u64 {
    let set = (std::env::var("RAEA_MEZZ_CAP_GB").ok().and_then(|v| v.parse::<f64>().ok()).unwrap_or(20.0) * 1e9) as u64;
    let room = (free_bytes().unwrap_or(0) + used_bytes()).saturating_sub(reserve());
    set.min(room)
}

/// The bytes the copies hold now.
fn used_bytes() -> u64 {
    std::fs::read_dir(dir()).map(|rd| rd.flatten().filter_map(|e| e.metadata().ok()).map(|m| m.len()).sum()).unwrap_or(0)
}

/// The free space on the cache folder's volume (statvfs: the blocks an unprivileged writer may take).
#[cfg(unix)]
fn free_bytes() -> Option<u64> {
    let d = dir();
    let probe = if d.exists() { d } else { d.parent()?.to_path_buf() };
    let c = std::ffi::CString::new(probe.as_os_str().as_encoded_bytes()).ok()?;
    let mut s: libc::statvfs = unsafe { std::mem::zeroed() };
    if unsafe { libc::statvfs(c.as_ptr(), &mut s) } != 0 {
        return None;
    }
    Some(s.f_bavail as u64 * s.f_frsize as u64)
}

#[cfg(not(unix))]
fn free_bytes() -> Option<u64> {
    None
}

/// The folder kept under its cap (cap_bytes), the least recently used copies removed first.
fn prune() {
    let cap = cap_bytes() as f64;
    let Ok(rd) = std::fs::read_dir(dir()) else { return };
    let mut copies: Vec<(SystemTime, u64, PathBuf)> = rd
        .flatten()
        .filter(|e| !e.path().to_string_lossy().contains(".partial."))
        .filter_map(|e| {
            let m = e.metadata().ok()?;
            Some((m.modified().ok()?, m.len(), e.path()))
        })
        .collect();
    copies.sort();
    let mut total: u64 = copies.iter().map(|c| c.1).sum();
    for (_, len, p) in copies {
        if (total as f64) <= cap {
            break;
        }
        if std::fs::remove_file(&p).is_ok() {
            total -= len;
        }
    }
}

/// A source read through copies of the ranges an edit uses: a time inside a range is the copy's frame at the time less
/// the range's start (the copy's time t is the source's start + t), anything else the source's own. Times, presentation
/// times and frame indices stay the source's. `open` opens a copy, the first time a time inside it is asked for.
pub struct Ranged<D> {
    pub src: D,
    pub ranges: Vec<(RangeCopy, Option<D>)>,
    pub open: fn(&Path) -> R<D>,
}

/// The decoder for `src` (opened by `open`): its complete copy when there is one (every frame a keyframe, so a seek
/// decodes one frame); else the source with the ranges that have copies read from them; else the source. Every decoder
/// opened on a source decides the same way, so a render and a play opened at the same moment read the same pixels.
pub fn open<D: VideoDecoder>(src: &Path, open: fn(&Path) -> R<D>, frame_secs: impl FnOnce() -> Option<f64>) -> R<Ranged<D>> {
    let whole = resolve(src);
    if whole.as_path() != src {
        return Ok(Ranged { src: open(&whole)?, ranges: Vec::new(), open });
    }
    let ranges = ranges(src, frame_secs).into_iter().map(|r| (r, None)).collect();
    Ok(Ranged { src: open(src)?, ranges, open })
}

impl<D: VideoDecoder> Ranged<D> {
    /// The range copy covering `t`, opened if it wasn't: its start and decoder.
    fn copy_at(&mut self, t: f64) -> R<Option<(f64, &mut D)>> {
        let eps = 1e-6;
        let Some(i) = self.ranges.iter().position(|(r, _)| t >= r.start - eps && t < r.end - eps) else { return Ok(None) };
        let (r, dec) = &mut self.ranges[i];
        if dec.is_none() {
            *dec = Some((self.open)(&r.path)?);
        }
        Ok(Some((r.start, dec.as_mut().unwrap())))
    }
}

impl<D: VideoDecoder> VideoDecoder for Ranged<D> {
    fn info(&self) -> &Info {
        self.src.info()
    }
    fn frame_at(&mut self, t: f64) -> R<Frame> {
        match self.copy_at(t)? {
            Some((s, d)) => d.frame_at(t - s).map(|f| Frame { pts: f.pts + s, ..f }),
            None => self.src.frame_at(t),
        }
    }
    fn frame_nv12_at(&mut self, t: f64) -> R<Frame> {
        match self.copy_at(t)? {
            Some((s, d)) => d.frame_nv12_at(t - s).map(|f| Frame { pts: f.pts + s, ..f }),
            None => self.src.frame_nv12_at(t),
        }
    }
    #[cfg(target_vendor = "apple")]
    fn frame_surface_at(&mut self, t: f64) -> R<Option<crate::SurfaceFrame>> {
        match self.copy_at(t)? {
            Some((s, d)) => d.frame_surface_at(t - s).map(|f| f.map(|f| crate::SurfaceFrame { pts: f.pts + s, ..f })),
            None => self.src.frame_surface_at(t),
        }
    }
    #[cfg(target_vendor = "apple")]
    fn frame_surface_rgb_at(&mut self, t: f64) -> R<Option<crate::SurfaceFrame>> {
        match self.copy_at(t)? {
            Some((s, d)) => d.frame_surface_rgb_at(t - s).map(|f| f.map(|f| crate::SurfaceFrame { pts: f.pts + s, ..f })),
            None => self.src.frame_surface_rgb_at(t),
        }
    }
    // the source's own frame index (its presentation times, nothing decoded): a range's first and last count the
    // track's minimum frame duration, which a variable-rate phone clip runs finer than its frames (1/92 s at 30 fps)
    fn index_at(&mut self, t: f64) -> Option<usize> {
        self.src.index_at(t)
    }
    // the source's keyframes stand only while no range reads from a copy
    fn keyframes(&self) -> Option<Vec<usize>> {
        if self.ranges.is_empty() { self.src.keyframes() } else { None }
    }
}
