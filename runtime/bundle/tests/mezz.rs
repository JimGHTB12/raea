//! An optimized range copy on whichever platform the build is (docs/ports.feature PT4): ProRes through VideoToolbox on
//! a Mac, all-intra H.264 through the soft backend elsewhere. A long-GOP clip (openh264's one keyframe) gets a copy of a
//! range, a second ask inside it reuses that copy, the export cache's key changes once the copy is there, and a decoder
//! opened on the clip shows the same picture inside the range as the clip itself.

use std::path::Path;

use raea_io_media::{EncodeOpts, mezz};

/// `n` frames of a gradient that moves a pixel a frame, at 30 fps
fn clip(path: &Path, w: u32, h: u32, n: usize) {
    let opts = EncodeOpts { width: w, height: h, fps: 30.0, kbps: 800, quality: None, nv12: false, realtime: false };
    let mut enc = raea_io_media::soft::h264_encoder(path, &opts, None).unwrap();
    for k in 0..n {
        let mut rgba = vec![255u8; (w * h * 4) as usize];
        for y in 0..h as usize {
            for x in 0..w as usize {
                let i = (y * w as usize + x) * 4;
                rgba[i] = ((x + k) * 4 % 256) as u8;
                rgba[i + 1] = (y * 3 % 256) as u8;
                rgba[i + 2] = (k * 5 % 256) as u8;
            }
        }
        enc.push(&rgba).unwrap();
    }
    enc.finish(None).unwrap();
}

fn frame(src: &Path, t: f64) -> raea_io_media::Frame {
    let mut dec = raea_io_media::preferred().into_iter().find_map(|b| b.open_video(src).ok()).expect("a backend reads the clip");
    dec.frame_at(t).unwrap()
}

#[test]
fn a_range_copy_is_made_reused_and_keyed() {
    let dir = std::env::temp_dir().join(format!("raea-mezz-test-{}", std::process::id()));
    let edit = dir.join("edit");
    std::fs::create_dir_all(&edit).unwrap();
    // SAFETY: set before anything reads them, on the test's own thread
    unsafe {
        std::env::set_var("RAEA_MEZZ_DIR", dir.join("mezz"));
        std::env::set_var("RAEA_EXPORT_CACHE", dir.join("exports"));
        std::env::set_var("RAEA_MEZZ_CAP_GB", "1");
        std::env::set_var("RAEA_MEZZ_RESERVE_GB", "0");
    }
    let src = edit.join("clip.mp4");
    clip(&src, 128, 96, 60);
    let (tsx, js, out) = (edit.join("edit.tsx"), edit.join("edit.js"), edit.join("out.mp4"));
    std::fs::write(&tsx, "export default () => null;\n").unwrap();
    std::fs::write(&js, "/* bundle */").unwrap();
    let before = raea_runtime_bundle::export_cache(&tsx, &js, &out, &[]).expect("a cache key");

    let probe = mezz::probe(&src).expect("a probe");
    assert!(probe.long_gop(), "the clip is long-GOP: {probe:?}");
    assert!(!probe.wants(), "under 1080p, so no whole copy");
    assert!(mezz::ranges(&src).is_empty());
    let made = mezz::make_range(&src, 0.5, 1.2).unwrap();
    assert!(made.is_file());
    let ranges = mezz::ranges(&src);
    assert_eq!(ranges.len(), 1, "{ranges:?}");
    assert!(ranges[0].start <= 0.5 + 1e-6 && ranges[0].end >= 1.2 - 1e-6, "{ranges:?}");
    assert_eq!(mezz::make_range(&src, 0.6, 1.0).unwrap(), made, "inside the range: the same copy");
    let after = raea_runtime_bundle::export_cache(&tsx, &js, &out, &[]).expect("a cache key");
    assert_ne!(before, after, "the key changes once the copy is there");

    // the picture inside the range, through the copy and from the clip alone
    let through = frame(&src, 0.8);
    unsafe { std::env::set_var("RAEA_MEZZ", "0") };
    let alone = frame(&src, 0.8);
    unsafe { std::env::remove_var("RAEA_MEZZ") };
    assert_eq!((through.width, through.height), (alone.width, alone.height));
    let s = raea_io_media::ssim(&through.rgba, &alone.rgba, through.width as usize, through.height as usize);
    eprintln!("copy {} ({} bytes), frame 0.8 s SSIM {s:.4}", made.display(), std::fs::metadata(&made).map(|m| m.len()).unwrap_or(0));
    let _ = std::fs::remove_dir_all(&dir);
    assert!(s > 0.97, "SSIM {s}");
}
