//! The media verbs' command line (`raea media|ffmpeg|ffprobe`, and raea's shell, in process): the agent's media
//! work on raea-media (lib.rs and its files), and an HTML composition's render (html.rs).
//!
//!   media probe|still|sheet|audio|loudness|motion|cuts|freezes|proxy|lod|trim|concat FILE ...   media work, native (media help)
//!   ffmpeg ARGS | ffprobe ARGS        ffmpeg and ffprobe command lines read as the media verbs (no ffmpeg involved)
//!   render PAGE.html -o OUT.mp4|OUT.png|DIR [--at F,F] [--alpha] [--codec apple|soft|av1|ffmpeg] [--kbps K | --quality Q]
//!   shots [--size WxH] [--root DIR]  plain HTML pages' first frames as PNGs, over white: stdin lines `PAGE\tOUT.png`,
//!                                     one `ok OUT` or `err OUT: why` line each (bench/wpt's reftests)

use std::path::PathBuf;

/// A refusal: said on stderr, and the command ends with exit 1 (`cli` catches it; the process goes on).
struct Fail;
/// A command's own exit code (the media verbs answer with one).
struct Exit(i32);

fn fail(e: impl std::fmt::Display) -> ! {
    eprintln!("raea: {e}");
    std::panic::panic_any(Fail)
}

/// The command line as a function: the exit code, in this process (raea's shell runs it on the iPhone, where nothing
/// can be spawned).
pub fn command(args: &[String]) -> i32 {
    static HOOK: std::sync::Once = std::sync::Once::new();
    HOOK.call_once(|| {
        let prev = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            if info.payload().downcast_ref::<Fail>().is_none() && info.payload().downcast_ref::<Exit>().is_none() {
                prev(info);
            }
        }));
    });
    let a = args.to_vec();
    match std::panic::catch_unwind(move || run(a)) {
        Ok(()) => 0,
        Err(p) if p.downcast_ref::<Fail>().is_some() => 1,
        Err(p) => match p.downcast_ref::<Exit>() {
            Some(Exit(c)) => *c,
            None => std::panic::resume_unwind(p),
        },
    }
}

fn run(args: Vec<String>) {
    let flag = |n: &str| args.iter().position(|a| a == n).and_then(|i| args.get(i + 1)).cloned();
    let flag_on = |n: &str| args.iter().any(|a| a == n);
    let arg = |i: usize| args.get(i).cloned().unwrap_or_else(|| fail("missing argument (see apps/media/src/command.rs)"));
    match args.first().map(String::as_str).unwrap_or("") {
        "media" => std::panic::panic_any(Exit(crate::cli(&args[1..]))),
        "ffmpeg" => std::panic::panic_any(Exit(crate::ffmpeg_cli(&args[1..]))),
        "ffprobe" => std::panic::panic_any(Exit(crate::ffprobe_cli(&args[1..]))),
        "render" => {
            let out = PathBuf::from(flag("-o").unwrap_or_else(|| fail("render needs -o OUT.mp4|DIR")));
            let frames = flag("--at").map(|at| at.split(',').map(|f| f.trim().parse().unwrap_or_else(|_| fail(format!("bad frame {f}")))).collect());
            let opts = raea_runtime_encode::RenderOpts { frames, codec: flag("--codec"), kbps: flag("--kbps").and_then(|k| k.parse().ok()), quality: flag("--quality").map(|q| raea_io_media::parse_quality(&q).unwrap_or_else(|e| fail(e))), ssim: flag_on("--ssim"), sound: None, ten_bit: flag_on("--10bit"), alpha: flag_on("--alpha") };
            if opts.alpha && out.extension().is_some_and(|x| x != "png") {
                fail("--alpha writes PNGs (OUT.png or a folder): video here is H.264 or HEVC, which hold no alpha");
            }
            raea_runtime_encode::render_file(&arg(1), &out, &opts).unwrap_or_else(|e| fail(e));
            if let Some(note) = raea_page::not_applied::said() {
                println!("{note}");
            }
        }
        "shots" => {
            let size = flag("--size").map(|s| s.split_once('x').and_then(|(w, h)| Some((w.parse().ok()?, h.parse().ok()?))).unwrap_or_else(|| fail(format!("bad --size {s} (WxH)"))));
            let (size, root) = (size.unwrap_or((800, 600)), flag("--root").map(PathBuf::from));
            // Blitz parses and lays out each page, Vello paints it on the GPU (docs/blitz-bevy.feature B1e)
            match flag("--page").as_deref() {
                Some("blitz") | None => raea_page::motion::shots(size, root.as_deref(), Some(&raea_io_home::home().join("core/fonts"))).unwrap_or_else(|e| fail(e)),
                Some(other) => fail(format!("--page {other}: only blitz (bevy_ui's page went with B5d)")),
            }
        }
        other => fail(format!("unknown command {other:?}: media, ffmpeg, ffprobe, render or shots (see apps/media/src/command.rs)")),
    }
}
