//! bevy_console (the declaration's `"calls": "console"`): run a console command from TypeScript and read what it
//! printed. The console's window is egui's and draws only with a renderer, but its commands are systems that read
//! `ConsoleCommandEntered` and reply with `PrintConsoleLine`, so a headless App runs them the same: a line is written
//! as the window would write it (split as a shell does), and every printed line is kept until read.
//!
//!   console/run       {line}  -> null: runs it from the next frame; an unknown command is an error naming the known
//!   console/output    {}      -> [line]: what the commands printed since the last read
//!   console/commands  {}      -> [name]: the registered commands (help, clear, exit and the game's own)

use bevy::prelude::*;
use bevy_console::{ConsoleCommandEntered, ConsoleConfiguration, ConsoleSet, PrintConsoleLine};
use bevy_remote::BrpResult;
use serde_json::{Value, json};

use crate::{fail, params};

pub fn calls(app: &mut App) {
    crate::methods!(app, "console/run" => run, "console/output" => output, "console/commands" => commands);
    app.init_resource::<Printed>().add_systems(Update, keep.after(ConsoleSet::Commands));
}

/// Every line the console's commands printed, until console/output reads them
#[derive(Resource, Default)]
struct Printed(Vec<String>);

fn keep(mut lines: MessageReader<PrintConsoleLine>, mut printed: ResMut<Printed>) {
    for l in lines.read() {
        printed.0.push(l.line.clone());
    }
}

#[derive(serde::Deserialize)]
struct Line {
    line: String,
}

fn run(In(p): In<Option<Value>>, world: &mut World) -> BrpResult {
    let Line { line } = params(p)?;
    let mut args = shlex::Shlex::new(&line).collect::<Vec<_>>();
    if args.is_empty() {
        return Err(fail("console/run: an empty line"));
    }
    let command_name = args.remove(0);
    let known: Vec<&str> = world.resource::<ConsoleConfiguration>().commands.keys().copied().collect();
    if !known.contains(&command_name.as_str()) {
        return Err(fail(format!("no console command {command_name} (known: {})", known.join(", "))));
    }
    world.write_message(ConsoleCommandEntered { command_name, args });
    Ok(Value::Null)
}

fn output(In(_): In<Option<Value>>, world: &mut World) -> BrpResult {
    Ok(json!(std::mem::take(&mut world.resource_mut::<Printed>().0)))
}

fn commands(In(_): In<Option<Value>>, world: &mut World) -> BrpResult {
    Ok(json!(world.resource::<ConsoleConfiguration>().commands.keys().collect::<Vec<_>>()))
}
