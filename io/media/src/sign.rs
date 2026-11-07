//! Signing what raea writes, installed from outside this crate. Content Credentials and the watermark that carries an
//! export's id are raea's own service, so the engine holds only this seam: a program that signs (raea's own build)
//! installs a `Signer` before its first export, and every backend `backends()` hands out is wrapped by it, so each
//! encoder signs and marks what it finishes. With nothing installed, files are written as they are, unsigned.
use crate::Backend;
use std::path::Path;
use std::sync::OnceLock;

/// How raea made a file: its manifest's digital source type.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Made {
    /// drawn by raea's engine from code: renders, edits, games
    Created,
    /// made by a trained model (local foley and instruments)
    Generated,
}

/// What signs raea's exports.
pub trait Signer: Send + Sync {
    /// The backend whose encoders sign (and mark) each file they finish.
    fn wrap(&self, backend: Box<dyn Backend>) -> Box<dyn Backend>;
    /// A file written outside an encoder (a render's still, generated sound): signed in place, a PNG's pixels marked.
    fn sign_file(&self, path: &Path, made: Made);
}

static SIGNER: OnceLock<Box<dyn Signer>> = OnceLock::new();

/// Install the signer, once, before the first export. A second install is refused.
pub fn install(signer: Box<dyn Signer>) -> Result<(), String> {
    SIGNER.set(signer).map_err(|_| "raea-io-media: a signer is already installed".to_string())
}

/// Whether this program signs what it writes (a report says which: docs/targets.feature T1e).
pub fn signs() -> bool {
    SIGNER.get().is_some()
}

pub(crate) fn wrap(backend: Box<dyn Backend>) -> Box<dyn Backend> {
    match SIGNER.get() {
        Some(s) => s.wrap(backend),
        None => backend,
    }
}

/// Sign a file raea wrote outside an encoder (a still, generated sound), when a signer is installed.
pub fn sign_file(path: &Path, made: Made) {
    if let Some(s) = SIGNER.get() {
        s.sign_file(path, made);
    }
}
