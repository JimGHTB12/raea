mod face_tests {
    use crate::ingest::confirm_faces;

    fn seq(confs: &[f64]) -> Vec<(f64, Vec<([f64; 4], f64)>)> {
        confs.iter().enumerate().map(|(k, c)| (k as f64 / 4.0, if *c > 0.0 { vec![([0.4, 0.2, 0.2, 0.15], *c)] } else { vec![] })).collect()
    }

    fn visible(v: &serde_json::Value) -> Vec<bool> {
        v[0]["samples"].as_array().unwrap().iter().map(|s| s["visible"].as_bool().unwrap()).collect()
    }

    #[test]
    fn a_face_is_confirmed_held_through_a_dropout_and_let_go() {
        // enters weak, confirmed on the second strong sighting, one missed sample (held), then gone (off after 0.5 s)
        let v = confirm_faces(&seq(&[0.6, 0.9, 0.95, 0.0, 0.92, 0.0, 0.0, 0.0]), 4.0);
        assert_eq!(visible(&v), vec![false, false, true, true, true, true, false, false]);
    }

    #[test]
    fn a_one_sample_low_blip_never_shows() {
        let v = confirm_faces(&seq(&[0.0, 0.0, 0.6, 0.0, 0.0]), 4.0);
        assert!(visible(&v).iter().all(|b| !b));
    }

    #[test]
    fn two_faces_keep_their_own_tracks() {
        let raw = vec![(0.0, vec![([0.1, 0.1, 0.2, 0.2], 1.0), ([0.6, 0.1, 0.2, 0.2], 1.0)]), (0.25, vec![([0.61, 0.1, 0.2, 0.2], 1.0), ([0.11, 0.1, 0.2, 0.2], 1.0)])];
        let v = confirm_faces(&raw, 4.0);
        assert_eq!(v.as_array().unwrap().len(), 2);
        assert_eq!(v[0]["samples"][1]["conf"], 1.0);
    }
}

mod tests {
    use crate::ingest::rival;

    /// a 64 x 64 map: a floor of 0.01 and Gaussian blobs (centre, strength)
    fn map(blobs: &[([f64; 2], f64)]) -> Vec<f32> {
        (0..64 * 64)
            .map(|i| {
                let (x, y) = (((i % 64) as f64 + 0.5) / 64.0, ((i / 64) as f64 + 0.5) / 64.0);
                (0.01 + blobs.iter().map(|(c, s)| s * (-((x - c[0]).powi(2) + (y - c[1]).powi(2)) / 0.003).exp()).sum::<f64>()) as f32
            })
            .collect()
    }

    #[test]
    fn one_place_has_no_rival() {
        let (_, r) = rival(&map(&[([0.5, 0.4], 1.0)]), 64, 64).unwrap();
        assert!(r < 0.1, "{r}");
    }

    #[test]
    fn two_equal_places_compete() {
        let (p, r) = rival(&map(&[([0.3, 0.3], 1.0), ([0.7, 0.7], 0.95)]), 64, 64).unwrap();
        assert!(r > 0.8, "{r}");
        assert!((p[0] - 0.3).abs() < 0.05 || (p[0] - 0.7).abs() < 0.05);
    }

    #[test]
    fn a_weaker_second_place_reads_weaker() {
        let (_, r) = rival(&map(&[([0.3, 0.3], 1.0), ([0.7, 0.7], 0.4)]), 64, 64).unwrap();
        assert!(r > 0.3 && r < 0.5, "{r}");
    }
}
