use super::*;

// a round trip: frames encoded to packets, the packets decoded with the encoder's own avcC, the same pictures back
#[test]
fn packets_round_trip() {
    let (w, h) = (320u32, 240u32);
    let mut enc = PacketEncoder::new(w, h, 30.0, 4000).unwrap();
    let mut packets = Vec::new();
    let frame = |i: usize| -> Vec<u8> {
        let mut v = vec![0u8; (w * h + w * (h / 2)) as usize];
        for y in 0..h as usize {
            for x in 0..w as usize {
                v[y * w as usize + x] = (16 + (x + i * 4) % 200) as u8;
            }
        }
        for b in &mut v[(w * h) as usize..] {
            *b = 128;
        }
        v
    };
    for i in 0..20 {
        packets.extend(enc.encode(&frame(i), i as f64 / 30.0, 1.0 / 30.0).unwrap());
    }
    packets.extend(enc.finish().unwrap());
    assert_eq!(packets.len(), 20);
    assert!(packets[0].key);
    let desc = enc.description().expect("an avcC");
    let mut dec = PacketDecoder::new("avc1", &desc).unwrap();
    let mut frames = Vec::new();
    for p in &packets {
        frames.extend(dec.decode(&p.data, p.pts, p.dur).unwrap());
    }
    frames.extend(dec.flush().unwrap());
    assert_eq!(frames.len(), 20);
    assert!(frames.windows(2).all(|f| f[0].pts < f[1].pts));
    // the luma ramp comes back within a few levels
    let want = frame(10);
    let got = &frames[10].nv12;
    let err: f64 = (0..(w * h) as usize).step_by(7).map(|i| (want[i] as f64 - got[i] as f64).abs()).sum::<f64>() / ((w * h) as f64 / 7.0);
    assert!(err < 4.0, "mean luma error {err}");
}
