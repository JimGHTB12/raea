//! ComfyUI both ways: a workflow (UI or API format) read into named nodes and lifted to a flow; a flow written back as
//! an API-format workflow; and a ComfyUI server (local, or hosted on Modal or any machine) as a place a step runs.

pub mod api_nodes;
pub mod lift;
pub mod needs;
pub mod read;
pub mod schema;
pub mod server;
pub mod view;
pub mod write;

use serde_json::Value as J;

use raea_io_gen_core::flow::Flow;

pub(crate) use raea_io_gen_core::flow::index as flow_index;

/// A workflow's JSON as a flow, with the schema this machine has.
pub fn import(v: &J) -> Result<lift::Lifted, String> {
    let schema = schema::load();
    let g = read::read(v, &schema)?;
    Ok(lift::lift(&g, &schema))
}

/// A flow as API-format JSON for ComfyUI's /prompt.
pub fn export(f: &Flow) -> Result<J, String> {
    write::write(f)
}
