//! #28 (3): `--quality` reaches openh264. It ignored EncodeOpts.quality, so draft and master gave byte-identical files
//! (Windows' and Linux's encoder); now quality scales its bitrate (quality_kbps).
#![cfg(feature = "h264")]

use raea_io_codec::EncodeOpts;
use raea_io_soft::h264_encoder;

const W: u32 = 640;
const H: u32 = 360;

/// 90 frames (three seconds, so the rate outweighs the first frame) of a moving pattern of soft rings: detailed enough that the bitrate decides the size, not noise that
/// every rate leaves at openh264's coarsest quantiser
fn encode(quality: Option<f32>) -> u64 {
    let dir = std::env::temp_dir().join(format!("raea-soft-quality-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let out = dir.join(format!("{quality:?}.mp4"));
    let opts = EncodeOpts { width: W, height: H, fps: 30.0, kbps: 2000, quality, nv12: false, realtime: false };
    let mut enc = h264_encoder(&out, &opts, None).unwrap();
    for k in 0..90u32 {
        let px: Vec<u8> = (0..W * H).flat_map(|i| {
            let (x, y) = (i % W, i / W);
            let (dx, dy) = (x as i64 - 320 + k as i64 * 4, y as i64 - 180);
            let v = ((dx * dx + dy * dy) / 23 % 256) as u8;
            [v, v.wrapping_mul(3), 255 - v, 255]
        }).collect();
        enc.push(&px).unwrap();
    }
    enc.finish(None).unwrap();
    let n = std::fs::metadata(&out).unwrap().len();
    let _ = std::fs::remove_file(&out);
    n
}

#[test]
fn quality_changes_what_openh264_writes() {
    let (draft, master, rate) = (encode(Some(0.6)), encode(Some(0.95)), encode(Some(0.85)));
    // before, all three were the same file
    assert!(draft * 5 / 4 < rate && rate * 5 / 4 < master, "draft {draft}, high {rate} and master {master} bytes: a higher quality spends more");
}
