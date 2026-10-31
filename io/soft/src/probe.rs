//! A source's video track from its container alone (raea_io_codec::mezz's Probe): the top-level boxes are walked by
//! their headers and only `ftyp` and `moov` are read, so a 4K take's media data is never loaded to decide about it.

use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

use raea_io_codec::R;
use raea_io_codec::mezz::Probe;

/// The file's `ftyp` and `moov` boxes, back to back (what re_mp4 needs for the tracks' tables).
fn header_boxes(path: &Path) -> R<Vec<u8>> {
    let mut f = File::open(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let len = f.metadata().map_err(|e| e.to_string())?.len();
    let (mut at, mut out) = (0u64, Vec::new());
    while at + 8 <= len {
        let mut h = [0u8; 16];
        f.seek(SeekFrom::Start(at)).map_err(|e| e.to_string())?;
        f.read_exact(&mut h[..8]).map_err(|e| e.to_string())?;
        let size = match u32::from_be_bytes(h[..4].try_into().unwrap()) as u64 {
            0 => len - at,
            1 => {
                f.read_exact(&mut h[8..]).map_err(|e| e.to_string())?;
                u64::from_be_bytes(h[8..].try_into().unwrap())
            }
            s => s,
        };
        if size < 8 {
            return Err(format!("{}: a box of {size} bytes at {at}", path.display()));
        }
        if matches!(&h[4..8], b"ftyp" | b"moov") {
            let mut b = vec![0u8; size as usize];
            f.seek(SeekFrom::Start(at)).map_err(|e| e.to_string())?;
            f.read_exact(&mut b).map_err(|e| e.to_string())?;
            out.extend_from_slice(&b);
        }
        at += size;
    }
    Ok(out)
}

/// The source's first video track: its codec (the sample entry's code), size, rate, minimum frame duration, length and
/// keyframes' spacing (sync samples in decode order, counted to 9).
pub fn probe(path: &Path) -> R<Probe> {
    let bytes = header_boxes(path)?;
    let mp4 = re_mp4::Mp4::read_bytes(&bytes).map_err(|e| format!("{}: {e}", path.display()))?;
    let (_, t) = mp4.tracks().iter().find(|(_, t)| t.kind == Some(re_mp4::TrackKind::Video)).ok_or(format!("{}: no video track", path.display()))?;
    let codec: String = t.codec_string(&mp4).unwrap_or_default().chars().take(4).collect();
    let mut cts: Vec<i64> = t.samples.iter().map(|s| s.composition_timestamp).collect();
    cts.sort_unstable();
    let step = cts.windows(2).map(|w| w[1] - w[0]).filter(|d| *d > 0).min().unwrap_or(1);
    let span = cts.last().zip(cts.first()).map(|(b, a)| (b - a) as f64 / t.timescale as f64).unwrap_or(0.0);
    let fps = if cts.len() > 1 && span > 0.0 { (cts.len() - 1) as f64 / span } else { 30.0 };
    let (mut since, mut longest) = (0usize, 0usize);
    for s in &t.samples {
        since = if s.is_sync { 0 } else { since + 1 };
        longest = longest.max(since);
        if longest > 8 {
            break;
        }
    }
    Ok(Probe { codec, width: t.width as f64, height: t.height as f64, fps, frame: (step, t.timescale as i32), secs: t.duration as f64 / t.timescale as f64, longest_gop: longest })
}
