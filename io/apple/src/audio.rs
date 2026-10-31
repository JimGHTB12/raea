//! Sound on Apple: decoding a file's audio, AAC round trips and files, and whether a file plays.

use super::*;

/// AudioToolbox through AVAudioFile: float columns at the file's rate, encoder priming and padding removed.
pub(crate) fn decode_audio_pooled(path: &Path) -> R<Audio> {
    use objc2_avf_audio::{AVAudioFile, AVAudioPCMBuffer};
    unsafe {
        let url = NSURL::fileURLWithPath(&NSString::from_str(&path.display().to_string()));
        let file = AVAudioFile::initForReading_error(AVAudioFile::alloc(), &url)
            .map_err(|e| format!("AVAudioFile: {}", e.localizedDescription()))?;
        let fmt = file.processingFormat();
        let (rate, ch) = (fmt.sampleRate(), fmt.channelCount() as usize);
        let total = file.length().max(0) as usize;
        let mut channels = vec![Vec::with_capacity(total); ch];
        let chunk = 1 << 16;
        loop {
            let buf = AVAudioPCMBuffer::initWithPCMFormat_frameCapacity(
                AVAudioPCMBuffer::alloc(),
                &fmt,
                chunk,
            )
            .ok_or("AVAudioPCMBuffer")?;
            file.readIntoBuffer_error(&buf)
                .map_err(|e| format!("AVAudioFile read: {}", e.localizedDescription()))?;
            let n = buf.frameLength() as usize;
            if n == 0 {
                break;
            }
            let data = buf.floatChannelData();
            for (c, col) in channels.iter_mut().enumerate() {
                col.extend_from_slice(std::slice::from_raw_parts((*data.add(c)).as_ptr(), n));
            }
            if n < chunk as usize {
                break;
            }
        }
        Ok(Audio {
            rate: rate.round() as u32,
            channels,
        })
    }
}


/// AAC (AudioToolbox, through AVAudioFile) from the audio columns.
/// The sound through AudioToolbox's AAC and back (what a player will decode from the file): for mastering to the
/// encoded result, as AAC can put the true peak over a limiter's ceiling.
pub fn aac_round_trip(a: &Audio) -> R<Audio> {
    let p = std::env::temp_dir().join(format!("raea-aac-{}-{:?}.m4a", std::process::id(), std::thread::current().id()));
    write_aac(&p, a)?;
    // read back by Symphonia, what the mastering was measured with
    let back = raea_io_soft::decode_audio_symphonia(&p);
    let _ = std::fs::remove_file(&p);
    back
}

pub(crate) fn write_aac(path: &Path, a: &Audio) -> R<()> {
    use objc2_avf_audio::{
        AVAudioFile, AVAudioFormat, AVAudioPCMBuffer, AVEncoderBitRateKey, AVFormatIDKey,
        AVNumberOfChannelsKey, AVSampleRateKey,
    };
    let a = raea_io_codec::soft_resample(a, 48000);
    let ch = a.channels.len().clamp(1, 2);
    let _ = std::fs::remove_file(path);
    unsafe {
        let url = NSURL::fileURLWithPath(&NSString::from_str(&path.display().to_string()));
        let settings: Retained<NSDictionary<NSString, AnyObject>> = NSDictionary::from_slices(
            &[
                AVFormatIDKey.ok_or("key")?,
                AVSampleRateKey.ok_or("key")?,
                AVNumberOfChannelsKey.ok_or("key")?,
                AVEncoderBitRateKey.ok_or("key")?,
            ],
            &[
                &*NSNumber::new_u32(u32::from_be_bytes(*b"aac ")) as &AnyObject,
                &*num(48000.0) as &AnyObject,
                &*num(ch as f64) as &AnyObject,
                &*num(128000.0) as &AnyObject,
            ],
        );
        let file =
            AVAudioFile::initForWriting_settings_error(AVAudioFile::alloc(), &url, &settings)
                .map_err(|e| format!("AVAudioFile: {}", e.localizedDescription()))?;
        let fmt = AVAudioFormat::initStandardFormatWithSampleRate_channels(
            AVAudioFormat::alloc(),
            48000.0,
            ch as u32,
        )
        .ok_or("AVAudioFormat")?;
        let chunk = 48000usize;
        let mut i = 0;
        while i < a.len() {
            let n = chunk.min(a.len() - i);
            let buf = AVAudioPCMBuffer::initWithPCMFormat_frameCapacity(
                AVAudioPCMBuffer::alloc(),
                &fmt,
                n as u32,
            )
            .ok_or("AVAudioPCMBuffer")?;
            buf.setFrameLength(n as u32);
            let data = buf.floatChannelData();
            for c in 0..ch {
                let dst = std::slice::from_raw_parts_mut((*data.add(c)).as_ptr(), n);
                dst.copy_from_slice(&a.channels[c][i..i + n]);
            }
            file.writeFromBuffer_error(&buf)
                .map_err(|e| format!("AVAudioFile write: {}", e.localizedDescription()))?;
            i += n;
        }
    }
    Ok(())
}


/// Playback as QuickTime does it, without an app: AVFoundation's verdict (playable, readable), then every video frame
/// and every audio sample decoded through AVAssetReader (video to BGRA, audio to linear PCM).
pub fn playable(path: &Path) -> R<serde_json::Value> {
    objc2::rc::autoreleasepool(|_| unsafe {
        let a = asset(path);
        #[allow(deprecated)]
        let (playable, readable) = (a.isPlayable(), a.isReadable());
        #[allow(deprecated)]
        let duration = a.duration().seconds();
        let read_track = |media: &AVMediaType, settings: Option<&NSDictionary<NSString, AnyObject>>| -> R<(usize, usize)> {
            #[allow(deprecated)]
            let tracks = a.tracksWithMediaType(media);
            let Some(track) = tracks.firstObject() else { return Ok((0, 0)) };
            let reader = AVAssetReader::assetReaderWithAsset_error(&a).map_err(|e| format!("AVAssetReader: {}", e.localizedDescription()))?;
            let out = AVAssetReaderTrackOutput::assetReaderTrackOutputWithTrack_outputSettings(&track, settings);
            out.setAlwaysCopiesSampleData(false);
            reader.addOutput(&out);
            if !reader.startReading() {
                return Err(format!("AVAssetReader didn't start: {:?}", reader.error().map(|e| e.localizedDescription().to_string())));
            }
            let (mut buffers, mut samples) = (0usize, 0usize);
            while let Some(sb) = objc2::rc::autoreleasepool(|_| out.copyNextSampleBuffer()) {
                buffers += 1;
                samples += sb.num_samples() as usize;
            }
            if reader.status() == AVAssetReaderStatus::Failed {
                return Err(format!("decode failed: {:?}", reader.error().map(|e| e.localizedDescription().to_string())));
            }
            Ok((buffers, samples))
        };
        let bgra: Retained<NSDictionary<NSString, AnyObject>> =
            NSDictionary::from_slices(&[cf_key(kCVPixelBufferPixelFormatTypeKey)], &[&*NSNumber::new_u32(kCVPixelFormatType_32BGRA) as &AnyObject]);
        let pcm: Retained<NSDictionary<NSString, AnyObject>> = NSDictionary::from_slices(
            &[objc2_avf_audio::AVFormatIDKey.ok_or("key")?],
            &[&*NSNumber::new_u32(u32::from_be_bytes(*b"lpcm")) as &AnyObject],
        );
        let video = read_track(AVMediaTypeVideo.ok_or("video")?, Some(&bgra));
        let audio = read_track(AVMediaTypeAudio.ok_or("audio")?, Some(&pcm));
        Ok(serde_json::json!({
            "file": path.display().to_string(), "avfoundation_playable": playable, "avfoundation_readable": readable, "duration": duration,
            "video_frames_decoded": video.as_ref().map(|v| v.1).ok(), "video_error": video.err(),
            "audio_samples_decoded": audio.as_ref().map(|v| v.1).ok(), "audio_error": audio.err(),
        }))
    })
}
