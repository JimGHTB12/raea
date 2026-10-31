//! The verbs that run raea's local tools: sound (`raea sao`, `tokensynth`, `setup-audio`, `sounds`: foley and an
//! instrument's notes made by local models, and the open sample libraries), `raea splat` (footage trained into a
//! Gaussian splat) and `raea song` (a song's energy map, apps/tools/src/song.rs).

pub mod song;
pub mod sound;
pub mod splat;
