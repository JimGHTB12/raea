//! The AVAssetWriter encoder, and pixel buffers from a pool.

use super::*;

/// Hardware H.264 through AVAssetWriter (VideoToolbox), BGRA pixel buffers from the writer's pool.
pub(crate) struct Encoder {
    pub(crate) writer: Retained<AVAssetWriter>,
    pub(crate) input: Retained<AVAssetWriterInput>,
    pub(crate) adaptor: Retained<AVAssetWriterInputPixelBufferAdaptor>,
    pub(crate) video: std::path::PathBuf,
    pub(crate) out: std::path::PathBuf,
    pub(crate) opts: EncodeOpts,
    pub(crate) n: i64,
}

pub(crate) fn num(v: f64) -> Retained<NSNumber> {
    NSNumber::new_f64(v)
}

impl Encoder {
    pub(crate) fn new(out: &Path, o: &EncodeOpts) -> R<Encoder> {
        let video = out.with_extension("video.mp4");
        let _ = std::fs::remove_file(&video);
        unsafe {
            let url = NSURL::fileURLWithPath(&NSString::from_str(&video.display().to_string()));
            let writer = AVAssetWriter::assetWriterWithURL_fileType_error(
                &url,
                AVFileTypeMPEG4.ok_or("AVFileTypeMPEG4")?,
            )
            .map_err(|e| format!("AVAssetWriter: {}", e.localizedDescription()))?;
            let compression: Retained<NSDictionary<NSString, AnyObject>> =
                NSDictionary::from_slices(
                    &[
                        AVVideoAverageBitRateKey.ok_or("key")?,
                        AVVideoMaxKeyFrameIntervalKey.ok_or("key")?,
                        AVVideoAllowFrameReorderingKey.ok_or("key")?,
                    ],
                    // no B-frames: display order = decode order, like the edit's mezzanines
                    &[
                        &*num((o.kbps * 1000) as f64) as &AnyObject,
                        &*num((o.fps * 2.0).round()) as &AnyObject,
                        &*NSNumber::new_bool(false) as &AnyObject,
                    ],
                );
            let codec: &NSString = AVVideoCodecTypeH264.ok_or("h264")?;
            let settings: Retained<NSDictionary<NSString, AnyObject>> = NSDictionary::from_slices(
                &[
                    AVVideoCodecKey.ok_or("key")?,
                    AVVideoWidthKey.ok_or("key")?,
                    AVVideoHeightKey.ok_or("key")?,
                    AVVideoCompressionPropertiesKey.ok_or("key")?,
                ],
                &[
                    codec as &AnyObject,
                    &*num(o.width as f64) as &AnyObject,
                    &*num(o.height as f64) as &AnyObject,
                    &*compression as &AnyObject,
                ],
            );
            let input = AVAssetWriterInput::assetWriterInputWithMediaType_outputSettings(
                AVMediaTypeVideo.ok_or("video")?,
                Some(&settings),
            );
            input.setExpectsMediaDataInRealTime(false);
            let attrs: Retained<NSDictionary<NSString, AnyObject>> = NSDictionary::from_slices(
                &[
                    cf_key(kCVPixelBufferPixelFormatTypeKey),
                    cf_key(kCVPixelBufferWidthKey),
                    cf_key(kCVPixelBufferHeightKey),
                ],
                &[
                    // NV12 from the engine's GPU readback goes in as it is; RGBA is swizzled to BGRA
                    &*NSNumber::new_u32(if o.nv12 { kCVPixelFormatType_420YpCbCr8BiPlanarVideoRange } else { kCVPixelFormatType_32BGRA }) as &AnyObject,
                    &*num(o.width as f64) as &AnyObject,
                    &*num(o.height as f64) as &AnyObject,
                ],
            );
            let adaptor = AVAssetWriterInputPixelBufferAdaptor::assetWriterInputPixelBufferAdaptorWithAssetWriterInput_sourcePixelBufferAttributes(&input, Some(&attrs));
            writer.addInput(&input);
            if !writer.startWriting() {
                return Err(format!(
                    "AVAssetWriter didn't start: {:?}",
                    writer.error().map(|e| e.localizedDescription().to_string())
                ));
            }
            writer.startSessionAtSourceTime(CMTime::with_seconds(0.0, 600));
            Ok(Encoder {
                writer,
                input,
                adaptor,
                video,
                out: out.to_path_buf(),
                opts: o.clone(),
                n: 0,
            })
        }
    }
}

impl VideoEncoder for Encoder {
    fn push(&mut self, rgba: &[u8]) -> R<()> {
        objc2::rc::autoreleasepool(|_| self.push_pooled(rgba))
    }
    fn push_nv12(&mut self, nv12: &[u8], w: usize, h: usize) -> R<()> {
        if !self.opts.nv12 {
            return Err("apple encoder: made for RGBA (EncodeOpts.nv12 false)".into());
        }
        objc2::rc::autoreleasepool(|_| self.push_nv12_pooled(nv12, w, h))
    }
    fn finish(self: Box<Self>, audio: Option<&Audio>) -> R<EncodeStats> {
        objc2::rc::autoreleasepool(|_| self.finish_pooled(audio))
    }
}

impl Encoder {
    /// NV12 straight into a pool buffer of the same format: two plane copies, row by row (the pool's rows may be
    /// padded), no conversion.
    pub(crate) fn push_nv12_pooled(&mut self, nv12: &[u8], w: usize, h: usize) -> R<()> {
        unsafe {
            while !self.input.isReadyForMoreMediaData() {
                std::thread::sleep(std::time::Duration::from_micros(200));
            }
            let pool = self.adaptor.pixelBufferPool().ok_or("no pixel buffer pool")?;
            let pb = pool_buffer(&pool)?;
            CVPixelBufferLockBaseAddress(&pb, CVPixelBufferLockFlags(0));
            raea_io_codec::encode_convert_timed(|| {
                for (plane, rows, off) in [(0usize, h, 0usize), (1, h / 2, w * h)] {
                    let base = CVPixelBufferGetBaseAddressOfPlane(&pb, plane) as *mut u8;
                    let bpr = CVPixelBufferGetBytesPerRowOfPlane(&pb, plane);
                    for r in 0..rows {
                        std::ptr::copy_nonoverlapping(nv12.as_ptr().add(off + r * w), base.add(r * bpr), w);
                    }
                }
            });
            CVPixelBufferUnlockBaseAddress(&pb, CVPixelBufferLockFlags(0));
            let t = CMTime::with_seconds(self.n as f64 / self.opts.fps, 6000);
            if !self.adaptor.appendPixelBuffer_withPresentationTime(&pb, t) {
                return Err(format!("append failed: {:?}", self.writer.error().map(|e| e.localizedDescription().to_string())));
            }
        }
        self.n += 1;
        Ok(())
    }

    pub(crate) fn push_pooled(&mut self, rgba: &[u8]) -> R<()> {
        unsafe {
            while !self.input.isReadyForMoreMediaData() {
                std::thread::sleep(std::time::Duration::from_micros(200));
            }
            let pool = self
                .adaptor
                .pixelBufferPool()
                .ok_or("no pixel buffer pool")?;
            let pb = pool_buffer(&pool)?;
            CVPixelBufferLockBaseAddress(&pb, CVPixelBufferLockFlags(0));
            let (w, h, bpr) = (
                CVPixelBufferGetWidth(&pb),
                CVPixelBufferGetHeight(&pb),
                CVPixelBufferGetBytesPerRow(&pb),
            );
            let base = CVPixelBufferGetBaseAddress(&pb) as *mut u8;
            raea_io_codec::encode_convert_timed(|| {
                for y in 0..h {
                    let row = std::slice::from_raw_parts_mut(base.add(y * bpr), w * 4);
                    let src = &rgba[y * w * 4..(y + 1) * w * 4];
                    for x in 0..w {
                        row[x * 4] = src[x * 4 + 2];
                        row[x * 4 + 1] = src[x * 4 + 1];
                        row[x * 4 + 2] = src[x * 4];
                        row[x * 4 + 3] = 255;
                    }
                }
            });
            CVPixelBufferUnlockBaseAddress(&pb, CVPixelBufferLockFlags(0));
            let t = CMTime::with_seconds(self.n as f64 / self.opts.fps, 6000);
            if !self.adaptor.appendPixelBuffer_withPresentationTime(&pb, t) {
                return Err(format!(
                    "append failed: {:?}",
                    self.writer
                        .error()
                        .map(|e| e.localizedDescription().to_string())
                ));
            }
        }
        self.n += 1;
        Ok(())
    }
    pub(crate) fn finish_pooled(self: Box<Self>, audio: Option<&Audio>) -> R<EncodeStats> {
        unsafe {
            self.input.markAsFinished();
            let done = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
            let d2 = done.clone();
            let block =
                block2::RcBlock::new(move || d2.store(true, std::sync::atomic::Ordering::SeqCst));
            self.writer.finishWritingWithCompletionHandler(&block);
            while !done.load(std::sync::atomic::Ordering::SeqCst) {
                std::thread::sleep(std::time::Duration::from_millis(2));
            }
            if self.writer.status() != AVAssetWriterStatus::Completed {
                return Err(format!(
                    "AVAssetWriter: {:?}",
                    self.writer
                        .error()
                        .map(|e| e.localizedDescription().to_string())
                ));
            }
        }
        let audio_codec = match audio {
            Some(a) => {
                let m4a = self.out.with_extension("aac.m4a");
                write_aac(&m4a, a)?;
                raea_io_codec::remux::h264_aac(&self.video, &m4a, &self.out, self.opts.fps)?;
                let _ = std::fs::remove_file(&m4a);
                let _ = std::fs::remove_file(&self.video);
                "aac (AudioToolbox)".to_string()
            }
            None => {
                std::fs::rename(&self.video, &self.out).map_err(|e| e.to_string())?;
                String::new()
            }
        };
        Ok(EncodeStats {
            frames: self.n as usize,
            bytes: std::fs::metadata(&self.out).map(|m| m.len()).unwrap_or(0),
            video_codec: "h264 (VideoToolbox)".into(),
            audio_codec,
        })
    }
}


/// Frames the encoder may hold at once. AVAssetWriter's isReadyForMoreMediaData doesn't bound its queue when the input
/// isn't real-time (on the iPhone the hook rendered at 47 fps with 2.6 GB held, ~8 MB a frame), so the pixel buffer
/// pool does: past this many outstanding buffers a push waits for the encoder to give one back.
/// RAEA_ENC_INFLIGHT overrides it (0: unbounded, as before).
pub(crate) const FRAMES_IN_FLIGHT: i32 = 6;

/// A buffer from the encoder's pool, waiting while FRAMES_IN_FLIGHT are out.
pub(crate) unsafe fn pool_buffer(pool: &CVPixelBufferPool) -> R<objc2_core_foundation::CFRetained<CVPixelBuffer>> {
    use objc2_core_foundation::{CFDictionary, CFNumber, CFType};
    let key: &CFType = unsafe { objc2_core_video::kCVPixelBufferPoolAllocationThresholdKey };
    let cap = std::env::var("RAEA_ENC_INFLIGHT").ok().and_then(|v| v.parse().ok()).unwrap_or(FRAMES_IN_FLIGHT);
    if cap <= 0 {
        let mut pb: *mut CVPixelBuffer = std::ptr::null_mut();
        if unsafe { CVPixelBufferPool::create_pixel_buffer(None, pool, std::ptr::NonNull::new(&mut pb).unwrap()) } != 0 || pb.is_null() {
            return Err("CVPixelBufferPoolCreatePixelBuffer failed".into());
        }
        return Ok(unsafe { objc2_core_foundation::CFRetained::from_raw(std::ptr::NonNull::new(pb).unwrap()) });
    }
    let n = CFNumber::new_i32(cap);
    let aux = CFDictionary::<CFType, CFType>::from_slices(&[key], &[&n]);
    let t0 = std::time::Instant::now();
    loop {
        let mut pb: *mut CVPixelBuffer = std::ptr::null_mut();
        let r = unsafe { CVPixelBufferPool::create_pixel_buffer_with_aux_attributes(None, pool, Some(aux.as_opaque()), std::ptr::NonNull::new(&mut pb).unwrap()) };
        if r == objc2_core_video::kCVReturnWouldExceedAllocationThreshold {
            if t0.elapsed() > std::time::Duration::from_secs(10) {
                return Err("the encoder held its frames for 10 s".into());
            }
            std::thread::sleep(std::time::Duration::from_micros(200));
            continue;
        }
        if r != 0 || pb.is_null() {
            return Err(format!("CVPixelBufferPoolCreatePixelBufferWithAuxAttributes: {r}"));
        }
        return Ok(unsafe { objc2_core_foundation::CFRetained::from_raw(std::ptr::NonNull::new(pb).unwrap()) });
    }
}
