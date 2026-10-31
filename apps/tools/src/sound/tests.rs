use super::*;

#[test]
fn notes_parse_as_tone_reads_them() {
    assert_eq!(midi_of("C4"), Some(60));
    assert_eq!(midi_of("A0"), Some(21));
    assert_eq!(midi_of("C#4"), Some(61));
    assert_eq!(midi_of("Cs4"), Some(61));
    assert_eq!(midi_of("Db4"), Some(61));
    assert_eq!(midi_of("Bb2"), Some(46));
    assert_eq!(midi_of("C-1"), Some(0));
    assert_eq!(midi_of("64"), Some(64));
    assert_eq!(midi_of("H2"), None);
    assert_eq!(note_name(61, false), "C#4");
    assert_eq!(note_name(61, true), "Db4");
    assert_eq!(note_file(66), "Fs4");
    assert!(parse_notes("C3, C4,C5").unwrap().iter().map(|n| n.1).eq([48, 60, 72]));
    assert!(parse_notes("C3,X9").is_err());
}

#[test]
fn the_cli_reads_its_flags_and_words() {
    let a: Vec<String> = ["sao", "a", "door", "slam", "--seconds", "1.5", "-o", "x.wav", "--seed", "3"].iter().map(|s| s.to_string()).collect();
    assert_eq!(positional(&a[1..], &["--seconds", "--seed", "--dir", "-o"]).join(" "), "a door slam");
    assert_eq!(number(&a, "--seconds", 2.0).unwrap(), 1.5);
    assert_eq!(number(&a, "--seed", 1.0).unwrap(), 3.0);
    assert_eq!(number(&a, "--velocity", 100.0).unwrap(), 100.0);
    assert!(number(&["--seconds".to_string(), "x".to_string()], "--seconds", 2.0).is_err());
    assert_eq!(js_num(2.0), "2");
    assert_eq!(js_num(0.5), "0.5");
}

#[test]
fn sample_names_give_their_note_and_dynamic() {
    assert_eq!(note_in("Marimba_hit_Outrigger_B2_med_01.wav"), Some(47));
    assert_eq!(note_in("NoSus/JHPiano_NoSus_Close_A#0_vl2_rr1.wav"), Some(22));
    assert_eq!(note_in("Ds1.mp3"), Some(27));
    assert_eq!(note_in("Clap_rr1.wav"), None);
    assert_eq!(dynamic("Marimba_hit_Outrigger_B2_med_01.wav"), Some(40));
    assert_eq!(dynamic("JHPiano_NoSus_Close_A#0_vl3_rr1.wav"), Some(3));
    assert!(is_first_take("Snare2_HitNS_v2_rr1_Mid.wav") && !is_first_take("Snare2_HitNS_v2_rr2_Mid.wav"));
    assert_eq!(url_path("Idiophones/Struck Idiophones/Claps/Clap_rr1.wav"), "Idiophones/Struck%20Idiophones/Claps/Clap_rr1.wav");
    assert_eq!(static_path(Path::new("/x/obj/media/sounds/fluid-marimba/C4.mp3")), "sounds/fluid-marimba/C4.mp3");
}

#[test]
fn the_catalog_has_its_libraries() {
    let c = catalog().unwrap();
    let ids: Vec<&str> = c["packs"].as_array().unwrap().iter().map(|p| p["id"].as_str().unwrap()).collect();
    assert_eq!(ids, ["fluid", "salamander", "tonejs-instruments", "vcsl"]);
    let fluid = &c["packs"][0];
    assert_eq!(files_of(fluid, &fluid["instruments"][0]).len(), 88, "A0-C8");
}
