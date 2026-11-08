//! Optional local graph sonification. Playback is explicit and never starts from a test.

#[cfg(target_os = "windows")]
use std::process::Child;

const SAMPLE_RATE: u32 = 22_050;
const BYTES_PER_SAMPLE: usize = 2;
const MAX_POINTS: usize = 100;
const POINT_MILLISECONDS: u32 = 50;
const FADE_MILLISECONDS: u32 = 5;
const LOW_FREQUENCY: f64 = 220.0;
const PITCH_OCTAVES: f64 = 2.0;
const PEAK_AMPLITUDE: f64 = 0.18;
const WAV_HEADER_BYTES: usize = 44;

#[derive(Default)]
pub(crate) struct GraphAudio {
    #[cfg(target_os = "windows")]
    child: Option<Child>,
}

impl GraphAudio {
    pub(crate) fn start(&mut self, samples: &[Option<f64>], volume: u32) -> Result<(), String> {
        if volume == 0 {
            return Err(
                "Graph sound is muted. Increase the speech and graph volume to hear it.".into(),
            );
        }
        let wav = trace_wav(samples, volume)?;
        self.stop()?;
        #[cfg(target_os = "windows")]
        {
            const SCRIPT: &str = concat!(
                "$ErrorActionPreference = 'Stop'; ",
                "[Console]::InputEncoding = [System.Text.UTF8Encoding]::new($false); ",
                "[Console]::OutputEncoding = [System.Text.UTF8Encoding]::new($false); ",
                "try { $bytes = [Convert]::FromBase64String([Console]::In.ReadToEnd()); ",
                "$stream = [System.IO.MemoryStream]::new($bytes, $false); ",
                "try { $player = [System.Media.SoundPlayer]::new($stream); ",
                "try { $player.Load(); $player.PlaySync() } finally { $player.Dispose() } } ",
                "finally { $stream.Dispose() } } ",
                "catch { [Console]::Error.WriteLine($_.Exception.Message); exit 1 }"
            );
            let payload = base64(&wav);
            self.child = Some(crate::speech::start_local_process(
                SCRIPT,
                payload.as_bytes(),
                "Graph sound",
            )?);
            Ok(())
        }
        #[cfg(not(target_os = "windows"))]
        {
            let _ = wav;
            Err("Built-in graph sound is available on Windows. The coordinate table and graph description remain available here.".into())
        }
    }

    pub(crate) fn stop(&mut self) -> Result<(), String> {
        #[cfg(target_os = "windows")]
        {
            crate::speech::stop_local_process(&mut self.child, "Graph sound")
        }
        #[cfg(not(target_os = "windows"))]
        {
            Ok(())
        }
    }

    pub(crate) fn poll(&mut self) -> Result<(), String> {
        #[cfg(target_os = "windows")]
        {
            crate::speech::poll_local_process(&mut self.child, "Graph sound")
        }
        #[cfg(not(target_os = "windows"))]
        {
            Ok(())
        }
    }

    pub(crate) fn is_playing(&self) -> bool {
        #[cfg(target_os = "windows")]
        {
            self.child.is_some()
        }
        #[cfg(not(target_os = "windows"))]
        {
            false
        }
    }
}

impl Drop for GraphAudio {
    fn drop(&mut self) {
        if let Err(error) = self.stop() {
            eprintln!("Graph sound cleanup: {error}");
        }
    }
}

fn trace_wav(samples: &[Option<f64>], volume: u32) -> Result<Vec<u8>, String> {
    if !(2..=MAX_POINTS).contains(&samples.len()) {
        return Err(format!(
            "Use 2 to {MAX_POINTS} sampled graph points for sound."
        ));
    }
    if volume > 100 {
        return Err("Choose a graph sound volume from 0 to 100.".into());
    }
    let mut minimum = f64::INFINITY;
    let mut maximum = f64::NEG_INFINITY;
    for value in samples.iter().flatten() {
        if !value.is_finite() {
            return Err("Graph sound needs finite sampled values; undefined points should be marked as gaps.".into());
        }
        minimum = minimum.min(*value);
        maximum = maximum.max(*value);
    }
    if !minimum.is_finite() {
        return Err("The selected graph has no finite values to play.".into());
    }
    // Normalize before subtraction to avoid overflow for large finite graph values.
    let normalizer = minimum.abs().max(maximum.abs()).max(1.0);
    let low = minimum / normalizer;
    let high = maximum / normalizer;
    let frames_per_point = (SAMPLE_RATE * POINT_MILLISECONDS / 1000) as usize;
    let fade_frames = (SAMPLE_RATE * FADE_MILLISECONDS / 1000) as usize;
    let byte_count = samples.len() * frames_per_point * BYTES_PER_SAMPLE;
    let mut output = Vec::with_capacity(WAV_HEADER_BYTES + byte_count);
    output.extend_from_slice(b"RIFF");
    output.extend_from_slice(&(36 + byte_count as u32).to_le_bytes());
    output.extend_from_slice(b"WAVEfmt ");
    output.extend_from_slice(&16_u32.to_le_bytes());
    output.extend_from_slice(&1_u16.to_le_bytes()); // PCM
    output.extend_from_slice(&1_u16.to_le_bytes()); // mono
    output.extend_from_slice(&SAMPLE_RATE.to_le_bytes());
    output.extend_from_slice(&(SAMPLE_RATE * BYTES_PER_SAMPLE as u32).to_le_bytes());
    output.extend_from_slice(&(BYTES_PER_SAMPLE as u16).to_le_bytes());
    output.extend_from_slice(&16_u16.to_le_bytes());
    output.extend_from_slice(b"data");
    output.extend_from_slice(&(byte_count as u32).to_le_bytes());
    let amplitude = PEAK_AMPLITUDE * f64::from(i16::MAX) * f64::from(volume) / 100.0;
    let mut phase = 0.0_f64;
    for point in samples {
        let frequency = point.map(|value| {
            let position = if high == low {
                0.5
            } else {
                ((value / normalizer - low) / (high - low)).clamp(0.0, 1.0)
            };
            LOW_FREQUENCY * 2.0_f64.powf(PITCH_OCTAVES * position)
        });
        for frame in 0..frames_per_point {
            let value = if let Some(frequency) = frequency {
                let fade_in = frame as f64 / fade_frames as f64;
                let fade_out = (frames_per_point - 1 - frame) as f64 / fade_frames as f64;
                let envelope = fade_in.min(fade_out).min(1.0);
                let value = (phase.sin() * amplitude * envelope).round() as i16;
                phase = (phase + std::f64::consts::TAU * frequency / f64::from(SAMPLE_RATE))
                    % std::f64::consts::TAU;
                value
            } else {
                0
            };
            output.extend_from_slice(&value.to_le_bytes());
        }
    }
    Ok(output)
}

#[cfg(any(target_os = "windows", test))]
fn base64(bytes: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let first = chunk[0];
        let second = chunk.get(1).copied().unwrap_or(0);
        let third = chunk.get(2).copied().unwrap_or(0);
        result.push(char::from(ALPHABET[usize::from(first >> 2)]));
        result.push(char::from(
            ALPHABET[usize::from((first & 3) << 4 | second >> 4)],
        ));
        result.push(if chunk.len() > 1 {
            char::from(ALPHABET[usize::from((second & 15) << 2 | third >> 6)])
        } else {
            '='
        });
        result.push(if chunk.len() > 2 {
            char::from(ALPHABET[usize::from(third & 63)])
        } else {
            '='
        });
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pcm(wav: &[u8]) -> Vec<i16> {
        wav[WAV_HEADER_BYTES..]
            .as_chunks::<2>()
            .0
            .iter()
            .map(|bytes| i16::from_le_bytes([bytes[0], bytes[1]]))
            .collect()
    }

    #[test]
    fn wave_header_and_playback_duration_are_bounded() {
        let wav = trace_wav(&vec![Some(1.0); MAX_POINTS], 100).unwrap();
        assert_eq!(&wav[..4], b"RIFF");
        assert_eq!(&wav[8..16], b"WAVEfmt ");
        assert_eq!(u16::from_le_bytes(wav[20..22].try_into().unwrap()), 1);
        assert_eq!(u16::from_le_bytes(wav[22..24].try_into().unwrap()), 1);
        assert_eq!(
            u32::from_le_bytes(wav[24..28].try_into().unwrap()),
            SAMPLE_RATE
        );
        assert_eq!(u16::from_le_bytes(wav[34..36].try_into().unwrap()), 16);
        assert_eq!(
            u32::from_le_bytes(wav[40..44].try_into().unwrap()) as usize,
            wav.len() - WAV_HEADER_BYTES
        );
        let seconds = pcm(&wav).len() as f64 / f64::from(SAMPLE_RATE);
        assert!(seconds <= 5.0 && seconds > 4.9);
        assert!(trace_wav(&[Some(1.0)], 80).is_err());
        assert!(trace_wav(&vec![Some(1.0); MAX_POINTS + 1], 80).is_err());
    }

    #[test]
    fn undefined_segments_are_silent_and_volume_and_fades_bound_amplitude() {
        let samples = [Some(-1.0), None, Some(1.0)];
        let loud = pcm(&trace_wav(&samples, 100).unwrap());
        let soft = pcm(&trace_wav(&samples, 25).unwrap());
        let muted = pcm(&trace_wav(&samples, 0).unwrap());
        let frames = (SAMPLE_RATE * POINT_MILLISECONDS / 1000) as usize;
        assert!(loud[frames..2 * frames].iter().all(|value| *value == 0));
        assert!(muted.iter().all(|value| *value == 0));
        for offset in [0, frames - 1, 2 * frames, 3 * frames - 1] {
            assert_eq!(loud[offset], 0);
        }
        let peak = |values: &[i16]| {
            values
                .iter()
                .map(|value| i32::from(*value).abs())
                .max()
                .unwrap()
        };
        assert!(peak(&loud) <= (PEAK_AMPLITUDE * f64::from(i16::MAX)).ceil() as i32);
        assert!((f64::from(peak(&soft)) / f64::from(peak(&loud)) - 0.25).abs() < 0.001);
    }

    #[test]
    fn rising_values_produce_higher_pitch_and_constant_values_remain_audible() {
        let values = pcm(&trace_wav(&[Some(-1.0), Some(1.0)], 80).unwrap());
        let half = values.len() / 2;
        let crossings = |slice: &[i16]| {
            slice
                .windows(2)
                .filter(|pair| pair[0] <= 0 && pair[1] > 0)
                .count()
        };
        assert!(crossings(&values[half..]) > crossings(&values[..half]) * 3);
        assert!(
            pcm(&trace_wav(&[Some(5.0); 3], 80).unwrap())
                .iter()
                .any(|value| *value != 0)
        );
        assert!(trace_wav(&[Some(-f64::MAX), Some(f64::MAX)], 80).is_ok());
    }

    #[test]
    fn invalid_traces_are_rejected_without_launching_playback() {
        assert!(trace_wav(&[None, None], 80).is_err());
        assert!(trace_wav(&[Some(f64::NAN), Some(0.0)], 80).is_err());
        assert!(trace_wav(&[Some(f64::INFINITY), Some(0.0)], 80).is_err());
        assert!(trace_wav(&[Some(0.0); 2], 101).is_err());
        let mut audio = GraphAudio::default();
        assert!(audio.start(&[Some(0.0); 2], 0).is_err());
        assert!(audio.start(&[None; 2], 80).is_err());
        assert!(audio.stop().is_ok());
        assert!(!audio.is_playing());
    }

    #[test]
    fn base64_encodes_binary_payload_without_executable_text() {
        for (input, expected) in [
            (b"".as_slice(), ""),
            (b"f", "Zg=="),
            (b"fo", "Zm8="),
            (b"foo", "Zm9v"),
            (&[0, 255, 128], "AP+A"),
        ] {
            assert_eq!(base64(input), expected);
        }
    }
}
