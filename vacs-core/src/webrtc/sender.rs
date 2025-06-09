use crate::audio::{EncodedAudioFrame, FRAME_DURATION_MS};
use anyhow::Result;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::sync::watch;
use webrtc::media::Sample;
use webrtc::track::track_local::track_local_static_sample::TrackLocalStaticSample;

pub struct Sender {
    shutdown_tx: watch::Sender<()>,
}

impl Sender {
    pub async fn new(
        track: Arc<TrackLocalStaticSample>,
        mut input_rx: mpsc::Receiver<EncodedAudioFrame>,
    ) -> Result<Self> {
        let (shutdown_tx, mut shutdown_rx) = watch::channel(());

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    biased;
                    _ = shutdown_rx.changed() => {
                        tracing::trace!("Shutdown signalled, stopping sending");
                        break;
                    }
                    frame = input_rx.recv() => {
                        match frame {
                            Some(frame) => {
                                let sample = Sample {
                                    data: frame,
                                    duration: std::time::Duration::from_millis(FRAME_DURATION_MS),
                                    ..Default::default()
                                };

                                if let Err(err) = track.write_sample(&sample).await {
                                    tracing::warn!(?err, "Failed to write sample to track");
                                }
                            }
                            None => {
                                break;
                            }
                        }
                    }
                }
            }
        });

        Ok(Self { shutdown_tx })
    }

    pub fn shutdown(&self) {
        let _ = self.shutdown_tx.send(());
    }
}

impl Drop for Sender {
    fn drop(&mut self) {
        self.shutdown();
    }
}
