//! fs.pick, fs.save, fs.read and fs.write: files the user picked or saved this run, by opaque id (the browser's model,
//! so the same app can run in a page). Ids last for the process and aren't kept; a file over CAP is refused both ways,
//! so base64 over the wire can't blow up memory.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD;
use serde_json::{Value, json};

use crate::{Done, Scripted, headless, scripted};

/// The most fs.read and fs.write move: 64 MB.
pub const CAP: u64 = 64 << 20;

static FILES: Mutex<(u64, Option<HashMap<u64, PathBuf>>)> = Mutex::new((0, None));

fn keep(p: PathBuf) -> u64 {
    let mut f = FILES.lock().unwrap();
    f.0 += 1;
    let id = f.0;
    f.1.get_or_insert_with(HashMap::new).insert(id, p);
    id
}

fn path(args: &Value) -> Result<PathBuf, String> {
    let id = args["id"].as_u64().ok_or("no id")?;
    FILES.lock().unwrap().1.as_ref().and_then(|m| m.get(&id).cloned()).ok_or(format!("no file {id} (pick or save one first)"))
}

fn too_big(n: u64) -> Result<(), String> {
    if n > CAP { Err(format!("the file is {n} bytes, over fs's 64 MB ({CAP} bytes)")) } else { Ok(()) }
}

fn described(p: &Path) -> Value {
    let id = keep(p.to_path_buf());
    json!({"id": id, "name": p.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default(), "size": std::fs::metadata(p).map(|m| m.len()).unwrap_or(0)})
}

#[cfg(not(target_os = "ios"))]
fn accept(args: &Value) -> Vec<String> {
    args["accept"].as_array().map(|a| a.iter().filter_map(Value::as_str).map(|e| e.trim_start_matches('.').to_string()).collect()).unwrap_or_default()
}

#[cfg(not(target_os = "ios"))]
fn dialog(args: &Value) -> rfd::AsyncFileDialog {
    let exts = accept(args);
    let d = rfd::AsyncFileDialog::new();
    if exts.is_empty() { d } else { d.add_filter("files", &exts) }
}

pub(crate) fn call(name: &str, args: &Value, done: Done) {
    match name {
        "fs.pick" => pick(args, done),
        "fs.save" => save(args, done),
        "fs.read" => done(read(args)),
        _ => done(write(args)),
    }
}

#[cfg_attr(target_os = "ios", expect(unused_variables, reason = "iOS builds no dialogs"))]
fn pick(args: &Value, done: Done) {
    if headless() {
        return done(match scripted(|s| if let Scripted::Pick(f) = s { Some(f.clone()) } else { None }) {
            Some(files) => Ok(Value::Array(files.iter().map(|p| described(&std::path::absolute(p).unwrap_or(p.clone()))).collect())),
            None => Err("fs.pick: headless, and no pick step answers it".into()),
        });
    }
    #[cfg(not(target_os = "ios"))]
    {
        let (multiple, d) = (args["multiple"] == json!(true), dialog(args));
        std::thread::spawn(move || {
            let files: Vec<PathBuf> = if multiple {
                pollster::block_on(d.pick_files()).unwrap_or_default().iter().map(|h| h.path().to_path_buf()).collect()
            } else {
                pollster::block_on(d.pick_file()).map(|h| vec![h.path().to_path_buf()]).unwrap_or_default()
            };
            done(Ok(Value::Array(files.iter().map(|p| described(p)).collect())));
        });
    }
}

#[cfg_attr(target_os = "ios", expect(unused_variables, reason = "iOS builds no dialogs"))]
fn save(args: &Value, done: Done) {
    if headless() {
        return done(match scripted(|s| if let Scripted::Save(f) = s { Some(f.clone()) } else { None }) {
            Some(p) => Ok(json!({"id": keep(std::path::absolute(&p).unwrap_or(p))})),
            None => Err("fs.save: headless, and no save step answers it".into()),
        });
    }
    #[cfg(not(target_os = "ios"))]
    {
        let d = dialog(args).set_file_name(args["name"].as_str().unwrap_or("untitled"));
        std::thread::spawn(move || done(Ok(pollster::block_on(d.save_file()).map_or(Value::Null, |h| json!({"id": keep(h.path().to_path_buf())})))));
    }
}

fn read(args: &Value) -> Result<Value, String> {
    let p = path(args)?;
    too_big(std::fs::metadata(&p).map_err(|e| format!("{}: {e}", p.display()))?.len())?;
    let bytes = std::fs::read(&p).map_err(|e| format!("{}: {e}", p.display()))?;
    if args["binary"] == json!(true) {
        Ok(json!(STANDARD.encode(bytes)))
    } else {
        String::from_utf8(bytes).map(|t| json!(t)).map_err(|_| format!("{} isn't text (read it with binary: true)", p.display()))
    }
}

fn write(args: &Value) -> Result<Value, String> {
    let p = path(args)?;
    let data = args["data"].as_str().unwrap_or("");
    let bytes = if args["binary"] == json!(true) { STANDARD.decode(data).map_err(|e| format!("fs.write's data isn't base64: {e}"))? } else { data.as_bytes().to_vec() };
    too_big(bytes.len() as u64)?;
    std::fs::write(&p, bytes).map(|()| Value::Null).map_err(|e| format!("{}: {e}", p.display()))
}
