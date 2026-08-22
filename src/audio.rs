use macroquad::audio::{
    load_sound_from_bytes, play_sound, stop_sound, PlaySoundParams, Sound,
};
use std::f32::consts::TAU;

const SAMPLE_RATE: u32 = 22_050;

pub struct GameAudio {
    music: Option<Sound>,
    bump: Option<Sound>,
    collect: Option<Sound>,
    win: Option<Sound>,
    pub music_enabled: bool,
}

impl GameAudio {
    pub async fn load() -> Self {
        let music_bytes = melody_wav();
        let bump_bytes = tone_wav(0.11, |t| {
            let envelope = (1.0 - t / 0.11).max(0.0);
            (TAU * (110.0 - 45.0 * t) * t).sin() * envelope * 0.35
        });
        let collect_bytes = tone_wav(0.42, |t| {
            let note = if t < 0.14 {
                523.25
            } else if t < 0.28 {
                659.25
            } else {
                783.99
            };
            (TAU * note * t).sin() * (1.0 - t / 0.55) * 0.28
        });
        let win_bytes = tone_wav(1.25, |t| {
            let notes = [392.0, 493.88, 587.33, 783.99, 987.77];
            let index = ((t / 0.24) as usize).min(notes.len() - 1);
            let local = t % 0.24;
            let envelope = (1.0 - local / 0.27).clamp(0.0, 1.0);
            (TAU * notes[index] * t).sin() * envelope * 0.25
        });

        Self {
            music: load_sound_from_bytes(&music_bytes).await.ok(),
            bump: load_sound_from_bytes(&bump_bytes).await.ok(),
            collect: load_sound_from_bytes(&collect_bytes).await.ok(),
            win: load_sound_from_bytes(&win_bytes).await.ok(),
            music_enabled: true,
        }
    }

    pub fn start_music(&self) {
        if self.music_enabled {
            if let Some(sound) = &self.music {
                play_sound(
                    sound,
                    PlaySoundParams {
                        looped: true,
                        volume: 0.16,
                    },
                );
            }
        }
    }

    pub fn toggle_music(&mut self) {
        self.music_enabled = !self.music_enabled;
        if let Some(sound) = &self.music {
            if self.music_enabled {
                play_sound(
                    sound,
                    PlaySoundParams {
                        looped: true,
                        volume: 0.16,
                    },
                );
            } else {
                stop_sound(sound);
            }
        }
    }

    pub fn play_bump(&self) {
        play_once(&self.bump, 0.20);
    }

    pub fn play_collect(&self) {
        play_once(&self.collect, 0.42);
    }

    pub fn play_win(&self) {
        play_once(&self.win, 0.48);
    }
}

fn play_once(sound: &Option<Sound>, volume: f32) {
    if let Some(sound) = sound {
        play_sound(
            sound,
            PlaySoundParams {
                looped: false,
                volume,
            },
        );
    }
}

fn melody_wav() -> Vec<u8> {
    let beat = 0.28;
    let notes = [
        220.00, 277.18, 329.63, 440.00, 369.99, 329.63, 277.18, 246.94, 220.00,
        329.63, 415.30, 493.88, 440.00, 369.99, 329.63, 277.18,
    ];
    let duration = beat * notes.len() as f32;
    tone_wav(duration, |t| {
        let index = ((t / beat) as usize).min(notes.len() - 1);
        let local = t % beat;
        let attack = (local / 0.035).min(1.0);
        let release = ((beat - local) / 0.07).clamp(0.0, 1.0);
        let envelope = attack * release;
        let frequency = notes[index];
        let lead = (TAU * frequency * t).sin();
        let bass = (TAU * (frequency * 0.5) * t).sin();
        (lead * 0.16 + bass * 0.09) * envelope
    })
}

/// Construye un WAV PCM mono de 16 bits completamente en memoria.
fn tone_wav(mut duration: f32, mut sample: impl FnMut(f32) -> f32) -> Vec<u8> {
    duration = duration.max(0.01);
    let sample_count = (duration * SAMPLE_RATE as f32) as u32;
    let data_size = sample_count * 2;
    let mut bytes = Vec::with_capacity((44 + data_size) as usize);

    bytes.extend_from_slice(b"RIFF");
    bytes.extend_from_slice(&(36 + data_size).to_le_bytes());
    bytes.extend_from_slice(b"WAVEfmt ");
    bytes.extend_from_slice(&16u32.to_le_bytes());
    bytes.extend_from_slice(&1u16.to_le_bytes());
    bytes.extend_from_slice(&1u16.to_le_bytes());
    bytes.extend_from_slice(&SAMPLE_RATE.to_le_bytes());
    bytes.extend_from_slice(&(SAMPLE_RATE * 2).to_le_bytes());
    bytes.extend_from_slice(&2u16.to_le_bytes());
    bytes.extend_from_slice(&16u16.to_le_bytes());
    bytes.extend_from_slice(b"data");
    bytes.extend_from_slice(&data_size.to_le_bytes());

    for index in 0..sample_count {
        let t = index as f32 / SAMPLE_RATE as f32;
        let value = (sample(t).clamp(-1.0, 1.0) * i16::MAX as f32) as i16;
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    bytes
}

