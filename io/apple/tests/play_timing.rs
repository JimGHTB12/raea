//! How long each frame of playback takes from the Mac decoder (frame_at at each frame's time, as the live player asks),
//! across the reader's recycle: RAEA_PLAY_TIMING=FILE.mp4 cargo test --release -p raea-io-apple --test play_timing -- --nocapture
#![cfg(target_vendor = "apple")]
use raea_io_codec::Backend;

#[test]
fn play_timing() {
    let Ok(path) = std::env::var("RAEA_PLAY_TIMING") else { return eprintln!("RAEA_PLAY_TIMING unset: skipped") };
    let mut d = raea_io_apple::Apple.open_video(std::path::Path::new(&path)).expect("open");
    let fps = d.info().fps;
    let n = ((d.info().duration * fps) as usize).min(420);
    let mut ms = Vec::with_capacity(n);
    for i in 0..n {
        let t0 = std::time::Instant::now();
        let f = d.frame_at((i as f64 + 0.5) / fps).expect("frame");
        // the frame showing at that time, never one from before a recycle or after it
        assert!((f.pts - i as f64 / fps).abs() < 0.5 / fps, "frame {i}: pts {:.4} for {:.4}", f.pts, i as f64 / fps);
        ms.push(t0.elapsed().as_secs_f64() * 1e3);
    }
    let mut sorted = ms.clone();
    sorted.sort_by(f64::total_cmp);
    let worst: Vec<_> = { let mut w: Vec<_> = ms.iter().enumerate().collect(); w.sort_by(|a, b| b.1.total_cmp(a.1)); w.into_iter().take(5).map(|(i, m)| format!("#{i} {m:.1} ms")).collect() };
    eprintln!("{path}: {n} frames at {fps} fps (budget {:.1} ms): median {:.1} ms, p95 {:.1} ms, worst {}", 1e3 / fps, sorted[n / 2], sorted[n * 95 / 100], worst.join(", "));
}
