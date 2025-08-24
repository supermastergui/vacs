use crate::app::state::{sealed, AppStateInner};
use crate::audio::manager::AudioManager;

pub trait AppStateAudioExt: sealed::Sealed {
    fn audio_manager(&mut self) -> &mut AudioManager;
}

impl AppStateAudioExt for AppStateInner {
    fn audio_manager(&mut self) -> &mut AudioManager {
        &mut self.audio_manager
    }
}