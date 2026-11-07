//! openh264's streams say how their pictures were converted (BT.709, limited range, as raea_io_codec::yuv converts
//! them): an untagged stream reads as BT.601 in ffmpeg and other players, which shifts reds and greens (#25). Both
//! encoders are checked, from the SPS each one writes.
#![cfg(feature = "h264")]

use raea_io_codec::EncodeOpts;
use raea_io_soft::{GopEncoder, h264_encoder};

const W: u32 = 64;
const H: u32 = 64;

/// A picture half green, half red, as the issue's repro draws.
fn frame() -> Vec<u8> {
    (0..W * H).flat_map(|i| if i % W < W / 2 { [0x22, 0xb5, 0x73, 255] } else { [0xe8, 0x17, 0x4a, 255] }).collect()
}

/// Reads an RBSP's bits, Exp-Golomb codes among them.
struct Bits<'a> {
    b: &'a [u8],
    at: usize,
}

impl Bits<'_> {
    fn bit(&mut self) -> u32 {
        let v = (self.b[self.at / 8] >> (7 - self.at % 8)) & 1;
        self.at += 1;
        v as u32
    }
    fn u(&mut self, n: u32) -> u32 {
        (0..n).fold(0, |v, _| v << 1 | self.bit())
    }
    fn ue(&mut self) -> u32 {
        let mut zeros = 0;
        while self.bit() == 0 {
            zeros += 1;
        }
        (1 << zeros) - 1 + self.u(zeros)
    }
}

/// An SPS's colour description (primaries, transfer, matrix, full range), or None when its VUI doesn't carry one.
/// `sps` is the NAL unit, header byte first. Enough of H.264's 7.3.2.1.1 for openh264's Constrained Baseline streams.
fn colour(sps: &[u8]) -> Option<(u32, u32, u32, bool)> {
    // emulation prevention bytes (00 00 03) out
    let mut rbsp = Vec::with_capacity(sps.len());
    for &x in &sps[1..] {
        if x == 3 && rbsp.ends_with(&[0, 0]) {
            continue;
        }
        rbsp.push(x);
    }
    let mut r = Bits { b: &rbsp, at: 0 };
    let profile = r.u(8);
    assert!(matches!(profile, 66 | 77 | 88), "profile {profile} has chroma fields this reader skips");
    r.u(16); // constraint flags, level
    r.ue(); // seq_parameter_set_id
    r.ue(); // log2_max_frame_num_minus4
    match r.ue() {
        0 => {
            r.ue();
        }
        1 => panic!("pic_order_cnt_type 1 isn't read here"),
        _ => {}
    }
    r.ue(); // max_num_ref_frames
    r.bit(); // gaps_in_frame_num_value_allowed_flag
    r.ue();
    r.ue(); // the size in macroblocks
    if r.bit() == 0 {
        r.bit(); // mb_adaptive_frame_field_flag
    }
    r.bit(); // direct_8x8_inference_flag
    if r.bit() == 1 {
        (0..4).for_each(|_| {
            r.ue();
        });
    }
    if r.bit() == 0 {
        return None; // no VUI
    }
    if r.bit() == 1 && r.u(8) == 255 {
        r.u(32); // sar_width, sar_height
    }
    if r.bit() == 1 {
        r.bit(); // overscan_appropriate_flag
    }
    if r.bit() == 0 {
        return None; // no video_signal_type
    }
    r.u(3); // video_format
    let full = r.bit() == 1;
    if r.bit() == 0 {
        return None; // no colour_description
    }
    Some((r.u(8), r.u(8), r.u(8), full))
}

/// The SPS among Annex B NAL units (00 00 01 or 00 00 00 01 before each).
fn sps_in_annex_b(s: &[u8]) -> Vec<u8> {
    let starts: Vec<usize> = (0..s.len().saturating_sub(3)).filter(|&i| s[i..i + 3] == [0, 0, 1]).map(|i| i + 3).collect();
    for (k, &a) in starts.iter().enumerate() {
        if s[a] & 0x1f == 7 {
            let mut end = starts.get(k + 1).map_or(s.len(), |&n| n - 3);
            while end > a && s[end - 1] == 0 {
                end -= 1;
            }
            return s[a..end].to_vec();
        }
    }
    panic!("no SPS in the stream")
}

/// The first SPS in an MP4's avcC box.
fn sps_in_mp4(mp4: &[u8]) -> Vec<u8> {
    let at = mp4.windows(4).position(|w| w == b"avcC").expect("no avcC box") + 4;
    let c = &mp4[at..];
    assert!(c[5] & 0x1f >= 1, "avcC holds no SPS");
    let n = u16::from_be_bytes([c[6], c[7]]) as usize;
    c[8..8 + n].to_vec()
}

/// BT.709 primaries, transfer and matrix (H.273 code 1 each), limited range.
const BT709: Option<(u32, u32, u32, bool)> = Some((1, 1, 1, false));

#[test]
fn an_exports_stream_is_tagged_bt709() {
    let out = std::env::temp_dir().join(format!("raea-io-soft-colour-{}.mp4", std::process::id()));
    let o = EncodeOpts { width: W, height: H, fps: 30.0, kbps: 2000, quality: None, nv12: false, realtime: false };
    let mut enc = h264_encoder(&out, &o, None).unwrap();
    for _ in 0..3 {
        enc.push(&frame()).unwrap();
    }
    enc.finish(None).unwrap();
    let mp4 = std::fs::read(&out).unwrap();
    let _ = std::fs::remove_file(&out);
    assert_eq!(colour(&sps_in_mp4(&mp4)), BT709);
}

#[test]
fn a_gops_stream_is_tagged_bt709() {
    let mut enc = GopEncoder::new(W, H, 30.0, 24).unwrap();
    let (first, key) = enc.push(&frame()).unwrap();
    assert!(key);
    assert_eq!(colour(&sps_in_annex_b(&first)), BT709);
}
