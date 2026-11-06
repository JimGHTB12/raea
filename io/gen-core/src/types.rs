//! The flow ops as TypeScript (`raea gen types`): sdk/gen/src/ops.gen.ts is written from OPS, so what raea/gen lets an
//! agent write is what the runner checks.

use crate::flow::OPS;

/// An input's TypeScript type, by its name.
fn ty(name: &str) -> &'static str {
    match name {
        "prompt" | "negative" | "text" | "system" | "aspect_ratio" | "resolution" | "sampler" | "scheduler" | "voice" | "path" | "url" | "output" => "string",
        "fit" => "\"stretch\" | \"cover\" | \"contain\"",
        "direction" => "\"right\" | \"down\" | \"left\" | \"up\"",
        "x" | "y" | "megapixels" => "number",
        "width" | "height" | "seed" | "steps" | "cfg" | "seconds" | "fps" | "factor" | "strength" | "speed" => "number",
        "audio" | "texture" => "boolean",
        "image" | "start_image" | "end_image" | "video" | "mask" => "Ref",
        "refs" | "images" => "Ref[]",
        "weights" => "string | string[]",
        "loras" => "{ name: string; strength?: number }[]",
        "nodes" => "Record<string, { class_type: string; inputs: Record<string, unknown> }>",
        "bind" => "Record<string, Ref>",
        "slots" => "number[]",
        _ => "unknown",
    }
}

/// A name as a TypeScript type name: image.generate → ImageGenerate.
fn pascal(op: &str) -> String {
    op.split(['.', '_']).map(|w| w[..1].to_uppercase() + &w[1..]).collect()
}

pub fn typescript() -> String {
    let mut out = String::from("// Written by `raea gen types` from io/gen/src/flow.rs OPS; don't edit by hand.\n\n/** Another step's output: `@step` (or `@step.N`). */\nexport type Ref = `@${string}`;\n/** A model's own setting, passed through as it is. */\nexport type Extra = { [setting: `x_${string}`]: unknown };\n\n");
    let mut ops = Vec::new();
    for (name, _, required, optional) in OPS {
        let t = pascal(name);
        out += &format!("export interface {t} extends Extra {{\n");
        for r in required.iter() {
            out += &format!("  {r}: {};\n", ty(r));
        }
        for o in optional.iter() {
            out += &format!("  {o}?: {};\n", ty(o));
        }
        out += "}\n\n";
        ops.push(format!("  \"{name}\": {t};"));
    }
    out += &format!("/** Every op and its inputs. */\nexport interface Ops {{\n{}\n}}\n\n", ops.join("\n"));
    let names: Vec<String> = OPS.iter().map(|o| format!("\"{}\"", o.0)).collect();
    out += &format!("/** The ops, for the builders. */\nexport const OP_NAMES = [{}] as const;\n", names.join(", "));
    out
}
