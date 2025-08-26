pub mod config;
mod device;
pub mod input;
pub mod output;
pub mod sources;
pub(crate) mod mixer;
pub mod stream;
mod dsp;

pub use device::Device;
pub use device::DeviceType;
pub use device::DeviceSelector;

use bytes::Bytes;

pub type EncodedAudioFrame = Bytes;

pub const TARGET_SAMPLE_RATE: u32 = 48_000;
pub const FRAME_DURATION_MS: u64 = 20;
const FRAME_SIZE: usize = TARGET_SAMPLE_RATE as usize * FRAME_DURATION_MS as usize / 1000;
