use super::*;

#[test]
fn a_frame_reads_as_its_light_and_how_much_it_changed() {
    let n = SIDE * SIDE;
    let mut bytes = vec![0u8; n];
    bytes.extend(vec![255u8; n]);
    bytes.extend(vec![255u8; n]);
    let p = of_gray(&bytes);
    assert_eq!(p.len(), 3);
    assert_eq!(p[0], Picture { luma: 0.0, motion: None });
    assert_eq!((p[1].luma, p[1].motion), (1.0, Some(1.0)), "black to white is all change");
    assert_eq!(p[2].motion, Some(0.0), "a held frame doesn't move");
    assert!(of_gray(&bytes[..n - 1]).is_empty(), "a part frame isn't one");
}

#[test]
fn contrast_is_the_text_against_the_half_least_like_it() {
    // white letters over black: half the pixels each
    let px: Vec<u8> = (0..100).flat_map(|i| if i % 2 == 0 { [255u8; 3] } else { [0u8; 3] }).collect();
    let r = ratio_behind(&px, lum(rgb_of("rgb(255, 255, 255)"))).unwrap();
    assert!((r - 21.0).abs() < 0.01, "{r}");
    // the same white over white is no contrast, whatever the letters
    let flat = vec![255u8; 300];
    assert!((ratio_behind(&flat, lum([255.0; 3])).unwrap() - 1.0).abs() < 1e-9);
    assert_eq!(rgb_of("rgba(10, 20, 30, 0.5)"), [10.0, 20.0, 30.0]);
    assert!(ratio_behind(&[], 1.0).is_none());
}

#[test]
fn ebur128s_summary_reads_as_loudness_and_peak() {
    let log = "[Parsed_ebur128_0] t: 1 M: -20\n[Parsed_ebur128_0] Summary:\n\n  Integrated loudness:\n    I:         -14.2 LUFS\n    Threshold: -24.4 LUFS\n\n  True peak:\n    Peak:       -1.3 dBFS\n";
    assert_eq!(of_ebur128(log), Some((-14.2, Some(-1.3))));
    assert_eq!(of_ebur128("Output #0, null: no audio"), None, "no sound, no loudness");
}

#[test]
fn a_sound_is_heard_where_the_mix_rises_and_not_under_a_louder_bed() {
    // 2 s at 16 kHz: silence, a tone from 0.50 s
    let rate = 16000;
    let tone: Vec<f32> = (0..2 * rate).map(|i| if i >= rate / 2 { 0.5 * (i as f32 * 0.1).sin() } else { 0.0 }).collect();
    let env = levels(&tone, rate / 100);
    let at = onset(&env, 0.5).unwrap();
    assert!((at - 0.5).abs() <= HOP + 1e-9, "heard at {at}");
    assert_eq!(onset(&env, 1.5), None, "nothing starts at 1.5 s: the tone is already playing");
    // the same tone under a bed 20 dB louder doesn't rise 6 dB
    let bed: Vec<f32> = tone.iter().enumerate().map(|(i, s)| s * 0.1 + 0.5 * (i as f32 * 0.37).sin()).collect();
    assert_eq!(onset(&levels(&bed, rate / 100), 0.5), None);
    assert_eq!(onset(&env, 30.0), None, "past the end");
    assert_eq!(onset(&env, 0.0), None, "the start of silence");
    // a real render's beep at 1.00 s (bench: an AAC mix read back): the encoder's pre-echo leaks 20 ms early, far
    // under the beep; it is heard where the beep is
    let mut aac = vec![-120.0f32; 98];
    aac.extend([-111.0, -102.9, -26.9, -24.1, -24.1, -23.9, -24.2]);
    aac.extend(vec![-24.0; 30]);
    assert_eq!(onset(&aac, 1.0), Some(1.0));
}
