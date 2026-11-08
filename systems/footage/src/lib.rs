//! raea's perception of footage (perception.feature): a clip read once, by content hash, the way an editor reads it:
//! its shots, cuts and camera, the words and faces on screen, its sound, and the creator's performance.

#[cfg(target_vendor = "apple")]
pub mod ingest;
pub mod shots;
pub mod store;
pub mod performance;
#[cfg(target_vendor = "apple")]
pub mod cuts;
