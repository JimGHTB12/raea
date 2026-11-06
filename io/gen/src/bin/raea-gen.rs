//! `raea-gen`: `raea gen` on its own (the same verbs, without raea's local models).
fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match raea_io_gen::cli::command(&args, None) {
        Ok(out) => println!("{out}"),
        Err(e) => {
            println!("REFUSED  {e}");
            std::process::exit(1);
        }
    }
}
