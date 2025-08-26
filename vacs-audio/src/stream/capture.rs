use crate::device::StreamDevice;
use crate::dsp::downmix_interleaved_to_mono;
use crate::{EncodedAudioFrame, FRAME_SIZE, TARGET_SAMPLE_RATE};
use anyhow::{Context, Result};
use bytes::Bytes;
use cpal::traits::StreamTrait;
use parking_lot::lock_api::Mutex;
use ringbuf::consumer::Consumer;
use ringbuf::producer::Producer;
use ringbuf::traits::Split;
use ringbuf::HeapRb;
use rubato::Resampler;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;
use tracing::instrument;

const MAX_OPUS_FRAME_SIZE: usize = 1275; // max size of an Opus frame according to RFC 6716 3.2.1.
const MIN_INPUT_BUFFER_SIZE: usize = 4096;
const RESAMPLER_BUFFER_SIZE: usize = 8192;
const RESAMPLER_BUFFER_WAIT: Duration = Duration::from_micros(500);

const INPUT_VOLUME_OPS_CAPACITY: usize = 16;
const INPUT_VOLUME_OPS_PER_DATA_CALLBACK: usize = 16;

type InputVolumeOp = Box<dyn Fn(&mut f32) + Send>;

pub struct CaptureStream {
    _stream: cpal::Stream,
    volume_ops: parking_lot::Mutex<ringbuf::HeapProd<InputVolumeOp>>,
    muted: Arc<AtomicBool>,
    cancel: CancellationToken,
    task: JoinHandle<()>,
}

impl CaptureStream {
    #[instrument(level = "debug", skip(tx), err)]
    pub fn start(
        device: StreamDevice,
        tx: mpsc::Sender<EncodedAudioFrame>,
        mut volume: f32,
        amp: f32,
    ) -> Result<Self> {
        tracing::debug!("Starting input capture stream");

        let muted = Arc::new(AtomicBool::new(false));
        let muted_clone = muted.clone();

        // buffer for ~100ms of input data
        let (mut input_prod, mut input_cons) =
            HeapRb::<f32>::new(((device.sample_rate() / 10) as usize).max(MIN_INPUT_BUFFER_SIZE))
                .split();

        let mut mono_buf: Vec<f32> = Vec::with_capacity(MIN_INPUT_BUFFER_SIZE);

        let stream = device
            .build_input_stream(
                move |input: &[f32], _| {
                    // downmix to mono if necessary
                    let mono: &[f32] = if device.config.channels > 1 {
                        downmix_interleaved_to_mono(
                            input,
                            device.config.channels as usize,
                            &mut mono_buf,
                        );
                        &mono_buf
                    } else {
                        input
                    };

                    let muted = muted_clone.load(Ordering::Relaxed);
                    let mut overflows = 0usize;
                    for &sample in mono {
                        // apply muting and push into input buffer to audio processing
                        if input_prod
                            .try_push(if muted { 0.0f32 } else { sample })
                            .is_err()
                        {
                            overflows += 1;
                            if overflows % 100 == 1 {
                                tracing::trace!(
                                    ?overflows,
                                    "Input buffer overflow (tail samples dropped)"
                                );
                            }
                        }
                    }
                    if overflows > 0 {
                        tracing::warn!(?overflows, "Dropped input samples during this callback");
                    }
                },
                |err| {
                    tracing::warn!(?err, "CPAL capture stream error");
                },
            )
            .context("Failed to build input stream")?;

        tracing::debug!("Starting capture on input stream");
        stream.play().context("Failed to play input stream")?;

        let cancel = CancellationToken::new();
        let cancel_clone = cancel.child_token();

        let (ops_prod, mut ops_cons) =
            HeapRb::<InputVolumeOp>::new(INPUT_VOLUME_OPS_CAPACITY).split();

        let mut resampler = device.resampler()?;

        let mut opus_framer = OpusFramer::new(tx)?;

        let task = tokio::runtime::Handle::current().spawn_blocking(move || {
            tracing::trace!("Input capture stream task started");

            let mut buf: Vec<f32> = Vec::with_capacity(RESAMPLER_BUFFER_SIZE);
            let mut resampler_in = vec![Vec::<f32>::with_capacity(FRAME_SIZE * 2)];

            while !cancel_clone.is_cancelled() {
                // apply any queued volume ops
                for _ in 0..INPUT_VOLUME_OPS_PER_DATA_CALLBACK {
                    if let Some(op) = ops_cons.try_pop() {
                        op(&mut volume);
                    } else {
                        break;
                    }
                }

                let gain = amp * volume;

                if let Some(resampler) = &mut resampler {
                    // buffer input data until we've reached enough to resample into the next frame
                    let need = resampler.input_frames_next();
                    while buf.len() < need {
                        if cancel_clone.is_cancelled() {
                            tracing::trace!("Input capture stream task cancelled");
                            break;
                        }
                        if let Some(sample) = input_cons.try_pop() {
                            buf.push(sample);
                        } else {
                            std::thread::sleep(RESAMPLER_BUFFER_WAIT);
                        }
                    }

                    if cancel_clone.is_cancelled() {
                        tracing::trace!("Input capture stream task cancelled");
                        break;
                    }
                    if buf.len() < need {
                        // canceled while waiting; exit
                        tracing::trace!("Did not receive enough input data to resample");
                        break;
                    }

                    resampler_in[0].clear();
                    resampler_in[0].extend_from_slice(&buf[..need]);
                    buf.drain(..need);

                    // resample the input data
                    let resampled = match resampler.process(&resampler_in, None) {
                        Ok(frames) => frames,
                        Err(err) => {
                            tracing::warn!(?err, "Failed to resample input");
                            continue;
                        }
                    };
                    let resampled = &resampled[0];

                    opus_framer.push_slice(resampled, gain);
                } else {
                    let mut stash: [f32; 1024] = [0.0; 1024];
                    let mut n = 0usize;

                    while let Some(sample) = input_cons.try_pop() {
                        if n == stash.len() {
                            opus_framer.push_slice(&stash[..n], gain);
                            n = 0;
                        }
                        stash[n] = sample;
                        n += 1;
                    }
                    if n > 0 {
                        opus_framer.push_slice(&stash[..n], gain);
                    } else {
                        std::thread::sleep(RESAMPLER_BUFFER_WAIT);
                    }
                }
            }

            tracing::trace!("Input capture stream task completed");
        });

        tracing::info!("Input capture stream started");
        Ok(Self {
            _stream: stream,
            volume_ops: Mutex::new(ops_prod),
            muted,
            cancel,
            task,
        })
    }

    #[instrument(level = "debug", skip(self))]
    pub async fn shutdown(self) {
        self.cancel.cancel();
        drop(self._stream);
        if let Err(err) = self.task.await {
            tracing::warn!(?err, "Input capture stream task failed");
        }
    }

    pub fn set_muted(&self, muted: bool) {
        self.muted.store(muted, Ordering::Relaxed);
    }

    pub fn is_muted(&self) -> bool {
        self.muted.load(Ordering::Relaxed)
    }

    pub fn set_volume(&self, volume: f32) {
        if self
            .volume_ops
            .lock()
            .try_push(Box::new(move |vol| *vol = volume.min(1.0)))
            .is_err()
        {
            tracing::warn!("Failed to queue volume op");
        }
    }
}

struct OpusFramer {
    frame: [f32; FRAME_SIZE],
    pos: usize,
    encoder: opus::Encoder,
    encoded: Vec<u8>,
    tx: mpsc::Sender<EncodedAudioFrame>,
}

impl OpusFramer {
    fn new(tx: mpsc::Sender<EncodedAudioFrame>) -> Result<Self> {
        let mut encoder = opus::Encoder::new(
            TARGET_SAMPLE_RATE,
            opus::Channels::Mono,
            opus::Application::Voip,
        )
        .context("Failed to create opus encoder")?;
        encoder
            .set_bitrate(opus::Bitrate::Max)
            .context("Failed to set opus bitrate")?;
        encoder
            .set_inband_fec(true)
            .context("Failed to set opus inband fec")?;
        encoder.set_vbr(false).context("Failed to set opus vbr")?;

        Ok(Self {
            frame: [0.0f32; FRAME_SIZE],
            pos: 0usize,
            encoder,
            encoded: vec![0u8; MAX_OPUS_FRAME_SIZE],
            tx,
        })
    }

    #[inline]
    fn push_slice(&mut self, samples: &[f32], gain: f32) {
        for &sample in samples {
            self.frame[self.pos] = (sample * gain).min(1.0);
            self.pos += 1;
            if self.pos == FRAME_SIZE {
                if let Ok(len) = self.encoder.encode_float(&self.frame, &mut self.encoded) {
                    let bytes = Bytes::copy_from_slice(&self.encoded[..len]);
                    if let Err(err) = self.tx.try_send(bytes) {
                        tracing::warn!(?err, "Failed to send input audio frame");
                    }
                } else {
                    tracing::warn!("Failed to encode input audio frame");
                }
                self.pos = 0;
            }
        }
    }
}
