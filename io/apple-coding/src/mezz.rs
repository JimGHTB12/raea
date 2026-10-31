//! Optimized media on Apple (lightspeed F4): the source's probe through AVFoundation and its intra-only ProRes 422 copy
//! through AVAssetExportSession. The index (where copies are, which are complete, the cap) is raea_io_codec::mezz;
//! raea_io_media::mezz decides when a copy is made.

use std::path::Path;

use objc2_av_foundation::*;
use objc2_foundation::{NSString, NSURL};

use raea_io_codec::R;
use raea_io_codec::mezz::Probe;

pub use raea_io_codec::mezz::{RangeCopy, dir, fingerprint, has, resolve};

fn asset(src: &Path) -> objc2::rc::Retained<AVURLAsset> {
    unsafe {
        let url = NSURL::fileURLWithPath(&NSString::from_str(&src.display().to_string()));
        AVURLAsset::URLAssetWithURL_options(&url, None)
    }
}

/// The source's video track as AVFoundation reads it: its codec, size, rate, frame duration and length, and its
/// keyframes' spacing (sync samples, up to 9 frames apart).
pub fn probe(src: &Path) -> Option<Probe> {
    unsafe {
        let asset = asset(src);
        let media = AVMediaTypeVideo?;
        #[allow(deprecated)]
        let tracks = asset.tracksWithMediaType(media);
        let track = tracks.firstObject()?;
        let descs = track.formatDescriptions();
        let d = descs.firstObject()?;
        let fd = &*(objc2::rc::Retained::as_ptr(&d) as *const objc2_core_media::CMFormatDescription);
        let codec = String::from_utf8_lossy(&fd.media_sub_type().to_be_bytes()).into_owned();
        let size = track.naturalSize();
        let fdur = track.minFrameDuration();
        #[allow(deprecated)]
        let secs = asset.duration().seconds().max(0.0);
        let mut longest = 0usize;
        if matches!(codec.as_str(), "avc1" | "avc3" | "hvc1" | "hev1")
            && let Some(cur) = track.makeSampleCursorAtFirstSampleInDecodeOrder()
        {
            let mut since = 0usize;
            loop {
                if cur.currentSampleSyncInfo().sampleIsFullSync.as_bool() {
                    since = 0;
                } else {
                    since += 1;
                    longest = longest.max(since);
                }
                if longest > 8 || cur.stepInDecodeOrderByCount(1) != 1 {
                    break;
                }
            }
        }
        Some(Probe { codec, width: size.width, height: size.height, fps: track.nominalFrameRate() as f64, frame: (fdur.value.max(1), fdur.timescale.max(1)), secs, longest_gop: longest })
    }
}

/// The source's copy (ProRes 422 video, and its audio as LPCM for a whole copy; video only for a range) written to
/// `part`: the whole file, or the frames `first..=last` (the probe's minimum frame duration: the copy's frame k is the
/// source's frame first + k). Blocks until done.
pub fn export(src: &Path, part: &Path, range: Option<(usize, usize)>, p: &Probe) -> R<()> {
    // the ProRes 422 export preset is macOS's only (iOS reads copies it's given, never makes one)
    #[cfg(not(target_os = "macos"))]
    return Err(format!("{}: optimized copies are made on macOS only ({part:?}, {range:?}, {p:?})", src.display()));
    #[cfg(target_os = "macos")]
    unsafe {
        use objc2_core_media::{CMTime, CMTimeRange};
        let asset = asset(src);
        let session = AVAssetExportSession::exportSessionWithAsset_presetName(&asset, AVAssetExportPresetAppleProRes422LPCM).ok_or("AVAssetExportSession: ProRes 422 isn't offered for this source")?;
        session.setOutputURL(Some(&NSURL::fileURLWithPath(&NSString::from_str(&part.display().to_string()))));
        session.setOutputFileType(AVFileTypeQuickTimeMovie);
        if let Some((first, last)) = range {
            let (fv, fts) = p.frame;
            session.setTimeRange(CMTimeRange { start: CMTime::new(first as i64 * fv, fts), duration: CMTime::new((last + 1 - first) as i64 * fv, fts) });
        }
        let (tx, rx) = std::sync::mpsc::channel::<()>();
        let done = block2::RcBlock::new(move || {
            let _ = tx.send(());
        });
        session.exportAsynchronouslyWithCompletionHandler(&done);
        rx.recv().map_err(|_| "the export never finished".to_string())?;
        if session.status() != AVAssetExportSessionStatus::Completed {
            let _ = std::fs::remove_file(part);
            let why = session.error().map(|e| e.localizedDescription().to_string()).unwrap_or_else(|| format!("status {:?}", session.status()));
            return Err(format!("{}: the ProRes copy failed: {why}", src.display()));
        }
        Ok(())
    }
}
