use serde::{Deserialize, Serialize};

pub(crate) mod commands;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioDevices {
    selected: String,
    default: String,
    all: Vec<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum VolumeType {
    Input,
    Output,
    Click,
    Chime
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioVolumes {
    input: f32,
    output: f32,
    click: f32,
    chime: f32,
}