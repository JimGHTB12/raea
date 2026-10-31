//! The Vision trait on open models (faces, person mask, saliency, text lines, flow), for perception off Apple.

#[cfg(feature = "vision-portable")]
pub mod portable;
#[cfg(feature = "face-portable")]
pub mod face_portable;
#[cfg(feature = "ocr-portable")]
pub mod ocr_portable;
pub mod flow_portable;
