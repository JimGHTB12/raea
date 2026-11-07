//! R1b: batches recorded from @raea/ui's React driver (js/record.ts, the R5 set in DOM mode) replay into the tree, and
//! after every batch the tree's shape equals what sdk/ui's strict test host held after the same batch.

use raea_rune_host::tree::{ROOT, Tree};
use serde_json::Value;

const APPS: [&str; 5] = ["counter", "settings", "list", "cards", "chat"];

#[test]
fn recorded_batches_replay_to_the_test_host_s_tree() {
    for app in APPS {
        let path = format!("{}/tests/recorded/{app}.json", env!("CARGO_MANIFEST_DIR"));
        let rec: Value = serde_json::from_str(&std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e} (run js/record.ts)"))).unwrap();
        let frames = rec["frames"].as_array().unwrap();
        assert!(frames.len() > 1, "{app}: a mount and at least one step");
        let mut tree = Tree::new();
        let mut kinds = std::collections::BTreeSet::new();
        for (i, f) in frames.iter().enumerate() {
            let batch = f["batch"].as_str().unwrap();
            for op in serde_json::from_str::<Value>(batch).unwrap()["ops"].as_array().unwrap() {
                kinds.insert(op["op"].as_str().unwrap().to_string());
            }
            tree.apply_json(batch).unwrap_or_else(|e| panic!("{app} batch {i} (step {}): {e}", f["step"]));
            assert_eq!(tree.shape(ROOT), f["shape"], "{app}: the tree after batch {i} (step {})", f["step"]);
        }
        eprintln!("{app}: {} batches replayed, ops {kinds:?}", frames.len());
    }
}
