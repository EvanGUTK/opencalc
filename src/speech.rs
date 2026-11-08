//! Local speech and token-aware math narration. No expression is interpolated into a command.

use serde::Serialize;

#[cfg(target_os = "windows")]
use std::{
    io::{Read, Write},
    os::windows::process::CommandExt,
    process::{Child, Command, ExitStatus, Stdio},
};

#[derive(Default)]
pub(crate) struct Speech {
    #[cfg(target_os = "windows")]
    child: Option<Child>,
    configuration: SpeechConfiguration,
}

const MAX_SPEECH_BYTES: usize = 32_768;
const MAX_NARRATION_BYTES: usize = 262_144;
const MAX_VOICE_NAME_BYTES: usize = 200;

#[derive(Clone, Debug, PartialEq, Eq)]
struct SpeechConfiguration {
    rate: i32,
    volume: u32,
    voice: String,
}

impl Default for SpeechConfiguration {
    fn default() -> Self {
        Self {
            rate: 0,
            volume: 80,
            voice: String::new(),
        }
    }
}

#[derive(Serialize)]
struct SpeechRequest<'a> {
    text: &'a str,
    rate: i32,
    volume: u32,
    voice: &'a str,
}

impl Speech {
    pub(crate) fn is_active(&self) -> bool {
        #[cfg(target_os = "windows")]
        {
            self.child.is_some()
        }
        #[cfg(not(target_os = "windows"))]
        {
            false
        }
    }

    pub(crate) fn configure(&mut self, rate: i32, volume: u32, voice: &str) -> Result<(), String> {
        if !(-5..=5).contains(&rate) {
            return Err("Choose a speech rate from -5 to 5.".into());
        }
        if volume > 100 {
            return Err("Choose a speech volume from 0 to 100.".into());
        }
        if voice.len() > MAX_VOICE_NAME_BYTES || voice.chars().any(char::is_control) {
            return Err("Choose a valid installed voice name.".into());
        }
        let configuration = SpeechConfiguration {
            rate,
            volume,
            voice: voice.to_owned(),
        };
        if self.configuration != configuration {
            self.stop()?;
            self.configuration = configuration;
        }
        Ok(())
    }

    pub(crate) fn speak(&mut self, text: &str) -> Result<(), String> {
        if text.trim().is_empty() {
            return Err("There is no text to read aloud.".into());
        }
        if text.len() > MAX_SPEECH_BYTES {
            return Err("The text is too long to read at once. Choose a shorter section.".into());
        }
        let payload = speech_payload(text, &self.configuration)?;
        self.poll()?;
        self.stop()?;
        #[cfg(target_os = "windows")]
        {
            const SCRIPT: &str = concat!(
                "$ErrorActionPreference = 'Stop'; ",
                "[Console]::InputEncoding = [System.Text.UTF8Encoding]::new($false); ",
                "[Console]::OutputEncoding = [System.Text.UTF8Encoding]::new($false); ",
                "try { Add-Type -AssemblyName System.Speech; ",
                "$request = [Console]::In.ReadToEnd() | ConvertFrom-Json; ",
                "$voice = New-Object System.Speech.Synthesis.SpeechSynthesizer; ",
                "try { $voice.Rate = [int]$request.rate; $voice.Volume = [int]$request.volume; ",
                "if (-not [string]::IsNullOrWhiteSpace([string]$request.voice)) { $voice.SelectVoice([string]$request.voice) }; ",
                "$voice.Speak([string]$request.text) } ",
                "finally { $voice.Dispose() } } ",
                "catch { [Console]::Error.WriteLine($_.Exception.Message); exit 1 }"
            );
            self.child = Some(start_local_process(SCRIPT, &payload, "Local speech")?);
            Ok(())
        }
        #[cfg(not(target_os = "windows"))]
        {
            let _ = payload;
            Err("Built-in speech is available on Windows. Use your system screen reader on this platform.".into())
        }
    }

    /// Reap completed speech and return any local speech-engine failure to the UI.
    pub(crate) fn poll(&mut self) -> Result<(), String> {
        #[cfg(target_os = "windows")]
        {
            poll_local_process(&mut self.child, "Local speech")
        }
        #[cfg(not(target_os = "windows"))]
        {
            Ok(())
        }
    }

    pub(crate) fn stop(&mut self) -> Result<(), String> {
        #[cfg(target_os = "windows")]
        {
            stop_local_process(&mut self.child, "Local speech")
        }
        #[cfg(not(target_os = "windows"))]
        {
            Ok(())
        }
    }
}

fn speech_payload(text: &str, configuration: &SpeechConfiguration) -> Result<Vec<u8>, String> {
    let narration = describe_math(text);
    if narration.len() > MAX_NARRATION_BYTES {
        return Err("The spoken description is too long. Choose a shorter section.".into());
    }
    serde_json::to_vec(&SpeechRequest {
        text: &narration,
        rate: configuration.rate,
        volume: configuration.volume,
        voice: &configuration.voice,
    })
    .map_err(|error| format!("Could not prepare local speech: {error}"))
}

pub(crate) fn installed_voices() -> Result<Vec<String>, String> {
    #[cfg(target_os = "windows")]
    {
        const SCRIPT: &str = concat!(
            "$ErrorActionPreference = 'Stop'; ",
            "[Console]::OutputEncoding = [System.Text.UTF8Encoding]::new($false); ",
            "try { Add-Type -AssemblyName System.Speech; $voice = New-Object System.Speech.Synthesis.SpeechSynthesizer; ",
            "try { $names = @($voice.GetInstalledVoices() | Where-Object { $_.Enabled } | ForEach-Object { $_.VoiceInfo.Name }); ",
            "ConvertTo-Json -InputObject $names -Compress } finally { $voice.Dispose() } } ",
            "catch { [Console]::Error.WriteLine($_.Exception.Message); exit 1 }"
        );
        let output = Command::new("powershell.exe")
            .args(["-NoProfile", "-NonInteractive", "-Command", SCRIPT])
            .creation_flags(0x0800_0000)
            .stdin(Stdio::null())
            .output()
            .map_err(|error| format!("Could not query installed voices: {error}"))?;
        if !output.status.success() {
            let detail = String::from_utf8_lossy(&output.stderr);
            return Err(format!(
                "Could not query installed voices: {}",
                detail.trim().chars().take(1000).collect::<String>()
            ));
        }
        parse_voice_names(&output.stdout)
    }
    #[cfg(not(target_os = "windows"))]
    {
        Err("Installed speech voices can be queried on Windows. Use your system screen reader on this platform.".into())
    }
}

#[cfg(any(target_os = "windows", test))]
fn parse_voice_names(json: &[u8]) -> Result<Vec<String>, String> {
    if json.len() > 65_536 {
        return Err("The installed voice list was too large.".into());
    }
    let text = std::str::from_utf8(json)
        .map_err(|_| "The voice list was not UTF-8 text.")?
        .trim_start_matches('\u{feff}');
    let mut voices: Vec<String> = serde_json::from_str(text)
        .map_err(|error| format!("Could not read the installed voice list: {error}"))?;
    if voices.len() > 128
        || voices.iter().any(|voice| {
            voice.is_empty()
                || voice.len() > MAX_VOICE_NAME_BYTES
                || voice.chars().any(char::is_control)
        })
    {
        return Err("The installed voice list contained invalid names.".into());
    }
    voices.sort();
    voices.dedup();
    Ok(voices)
}

#[cfg(target_os = "windows")]
pub(crate) fn start_local_process(
    script: &str,
    payload: &[u8],
    label: &str,
) -> Result<Child, String> {
    let mut child = Command::new("powershell.exe")
        .args(["-NoProfile", "-NonInteractive", "-Command", script])
        .creation_flags(0x0800_0000)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| format!("{label} could not start: {error}"))?;
    let written = child
        .stdin
        .take()
        .ok_or_else(|| format!("{label} input was unavailable."))
        .and_then(|mut input| {
            input
                .write_all(payload)
                .map_err(|error| format!("{label} input failed: {error}"))
        });
    if let Err(error) = written {
        let mut slot = Some(child);
        return match stop_local_process(&mut slot, label) {
            Ok(()) => Err(error),
            Err(cleanup) => Err(format!("{error}. {cleanup}")),
        };
    }
    Ok(child)
}

#[cfg(target_os = "windows")]
pub(crate) fn poll_local_process(slot: &mut Option<Child>, label: &str) -> Result<(), String> {
    let Some(child) = slot.as_mut() else {
        return Ok(());
    };
    let status = child
        .try_wait()
        .map_err(|error| format!("{label} status could not be checked: {error}"))?;
    if let Some(status) = status
        && let Some(mut child) = slot.take()
    {
        return playback_result(&mut child, status, label);
    }
    Ok(())
}

#[cfg(target_os = "windows")]
pub(crate) fn stop_local_process(slot: &mut Option<Child>, label: &str) -> Result<(), String> {
    let Some(mut child) = slot.take() else {
        return Ok(());
    };
    match child.try_wait() {
        Ok(Some(status)) => playback_result(&mut child, status, label),
        Ok(None) => {
            if let Err(error) = child.kill() {
                match child.try_wait() {
                    Ok(Some(status)) => return playback_result(&mut child, status, label),
                    _ => {
                        *slot = Some(child);
                        return Err(format!("{label} could not stop: {error}"));
                    }
                }
            }
            child.wait().map(|_| ()).map_err(|error| {
                *slot = Some(child);
                format!("{label} could not finish: {error}")
            })
        }
        Err(error) => {
            *slot = Some(child);
            Err(format!("{label} status could not be checked: {error}"))
        }
    }
}

impl Drop for Speech {
    fn drop(&mut self) {
        if let Err(error) = self.stop() {
            eprintln!("Local speech cleanup: {error}");
        }
    }
}

#[cfg(target_os = "windows")]
fn playback_result(child: &mut Child, status: ExitStatus, label: &str) -> Result<(), String> {
    if status.success() {
        return Ok(());
    }
    let mut detail = String::new();
    if let Some(output) = child.stderr.take() {
        output
            .take(2048)
            .read_to_string(&mut detail)
            .map_err(|error| format!("{label} failed; its error could not be read: {error}"))?;
    }
    if detail.trim().is_empty() {
        Err(format!("{label} failed ({status})."))
    } else {
        Err(format!("{label} failed: {}", detail.trim()))
    }
}

#[derive(Debug, PartialEq)]
enum Token {
    Word(String),
    Number(String),
    Symbol(char),
    Space,
}

/// Narrate complete math tokens, keeping prose words such as "cost" and "squarely" intact.
pub(crate) fn describe_math(text: &str) -> String {
    let tokens = tokenize(text);
    let mut output = String::new();
    let mut index = 0;
    let mut previous = None;
    while index < tokens.len() {
        let token = &tokens[index];
        let next = next_token(&tokens, index + 1);
        match token {
            Token::Word(word) => {
                let name = word.to_ascii_lowercase();
                let function = matches!(next.map(|(_, token)| token), Some(Token::Symbol('(')));
                let replacement = match name.as_str() {
                    "sin" if function => Some("sine of"),
                    "cos" if function => Some("cosine of"),
                    "tan" if function => Some("tangent of"),
                    "asin" if function => Some("inverse sine of"),
                    "acos" if function => Some("inverse cosine of"),
                    "atan" if function => Some("inverse tangent of"),
                    "sqrt" if function => Some("square root of"),
                    "cbrt" if function => Some("cube root of"),
                    "ln" if function => Some("natural logarithm of"),
                    "log" if function => Some("logarithm base ten of"),
                    "abs" if function => Some("absolute value of"),
                    "exp" if function => Some("e to the power of"),
                    "diff" if function => Some("derivative of"),
                    "integral" if function => Some("integral of"),
                    "root" if function => Some("root of"),
                    "ans" => Some("previous answer"),
                    "pi" => Some("pi"),
                    _ => None,
                };
                if let Some(replacement) = replacement {
                    phrase(&mut output, replacement);
                } else {
                    output.push_str(word);
                }
            }
            Token::Number(number) => {
                if let Some((mantissa, exponent)) = number.split_once(['e', 'E']) {
                    output.push_str(mantissa);
                    phrase(&mut output, "times ten to the power of");
                    if let Some(positive) = exponent.strip_prefix('+') {
                        output.push_str(positive);
                    } else if let Some(negative) = exponent.strip_prefix('-') {
                        phrase(&mut output, "negative");
                        output.push_str(negative);
                    } else {
                        output.push_str(exponent);
                    }
                } else {
                    output.push_str(number);
                }
            }
            Token::Space => output.push(' '),
            Token::Symbol('^') => {
                if let Some((power_index, Token::Number(power))) = next
                    && matches!(power.as_str(), "2" | "3")
                {
                    phrase(&mut output, if power == "2" { "squared" } else { "cubed" });
                    index = power_index;
                } else {
                    phrase(&mut output, "to the power of");
                }
            }
            Token::Symbol(symbol) => {
                let replacement = match symbol {
                    '+' => "plus",
                    '-' | '−' => {
                        if starts_operand(previous) {
                            "negative"
                        } else {
                            "minus"
                        }
                    }
                    '*' | '×' | '·' => "times",
                    '/' | '÷' => "divided by",
                    '=' => "equals",
                    '±' => "plus or minus",
                    '√' => "square root of",
                    'π' => "pi",
                    'θ' => "theta",
                    '²' => "squared",
                    '³' => "cubed",
                    '%' => "percent",
                    '(' => "open parenthesis",
                    ')' => "close parenthesis",
                    '[' => "open bracket",
                    ']' => "close bracket",
                    '∫' => "integral of",
                    '≤' => "less than or equal to",
                    '≥' => "greater than or equal to",
                    '≠' => "not equal to",
                    '∞' => "infinity",
                    '→' => "stored in",
                    '<' => "less than",
                    '>' => "greater than",
                    '!' if previous.is_some_and(is_math_value) => "factorial",
                    _ => {
                        output.push(*symbol);
                        index += 1;
                        previous = Some(token);
                        continue;
                    }
                };
                phrase(&mut output, replacement);
            }
        }
        if *token != Token::Space {
            previous = Some(&tokens[index]);
        }
        index += 1;
    }
    let words = output.split_whitespace().collect::<Vec<_>>().join(" ");
    tidy_punctuation(&words)
}

fn phrase(output: &mut String, text: &str) {
    output.push(' ');
    output.push_str(text);
    output.push(' ');
}

fn starts_operand(previous: Option<&Token>) -> bool {
    previous.is_none_or(|token| {
        matches!(
            token,
            Token::Symbol('(' | '[' | ',' | '=' | '+' | '-' | '−' | '*' | '/' | '^')
        )
    })
}

fn is_math_value(token: &Token) -> bool {
    matches!(token, Token::Number(_) | Token::Symbol(')' | ']' | 'π'))
        || matches!(token, Token::Word(word) if word.len() == 1 || word.eq_ignore_ascii_case("ans"))
}

fn next_token(tokens: &[Token], start: usize) -> Option<(usize, &Token)> {
    tokens
        .iter()
        .enumerate()
        .skip(start)
        .find(|(_, token)| **token != Token::Space)
}

fn tokenize(text: &str) -> Vec<Token> {
    let chars: Vec<char> = text.chars().collect();
    let mut tokens = Vec::new();
    let mut index = 0;
    while index < chars.len() {
        let start = index;
        let character = chars[index];
        if character.is_whitespace() {
            while chars.get(index).is_some_and(|c| c.is_whitespace()) {
                index += 1;
            }
            tokens.push(Token::Space);
        } else if character.is_ascii_alphabetic() {
            while chars.get(index).is_some_and(char::is_ascii_alphabetic) {
                index += 1;
            }
            tokens.push(Token::Word(chars[start..index].iter().collect()));
        } else if character.is_ascii_digit()
            || character == '.' && chars.get(index + 1).is_some_and(char::is_ascii_digit)
        {
            while chars
                .get(index)
                .is_some_and(|c| c.is_ascii_digit() || *c == '.')
            {
                index += 1;
            }
            if matches!(chars.get(index), Some('e' | 'E')) {
                let mut end = index + 1;
                if matches!(chars.get(end), Some('+' | '-')) {
                    end += 1;
                }
                let digits = end;
                while chars.get(end).is_some_and(char::is_ascii_digit) {
                    end += 1;
                }
                if end > digits {
                    index = end;
                }
            }
            tokens.push(Token::Number(chars[start..index].iter().collect()));
        } else {
            tokens.push(Token::Symbol(character));
            index += 1;
        }
    }
    tokens
}

fn tidy_punctuation(text: &str) -> String {
    let mut output = String::with_capacity(text.len());
    for character in text.chars() {
        if matches!(character, '.' | ',' | ':' | ';' | '!' | '?') && output.ends_with(' ') {
            output.pop();
        }
        output.push(character);
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn voice_configuration_is_validated_and_sent_only_as_json_data() {
        let mut speech = Speech::default();
        let voice = "Voice \"example\"; $(Get-Process)";
        speech.configure(-2, 35, voice).unwrap();
        let text = "Write-Output `\"example\"; $env:PATH and x^2";
        let payload = speech_payload(text, &speech.configuration).unwrap();
        let request: serde_json::Value = serde_json::from_slice(&payload).unwrap();
        assert_eq!(request["voice"], voice);
        assert_eq!(request["text"], describe_math(text));
        assert_eq!(request["rate"], -2);
        assert_eq!(request["volume"], 35);
        assert!(speech.configure(-6, 35, "").is_err());
        assert!(speech.configure(0, 101, "").is_err());
        assert!(speech.configure(0, 80, "invalid\nvoice").is_err());
        assert_eq!(
            speech.configuration.voice, voice,
            "invalid settings must not change the selected voice"
        );
    }

    #[test]
    fn installed_voice_json_keeps_unicode_and_validates_its_shape() {
        let names = parse_voice_names("[\"Zoe\",\"Español\",\"Zoe\"]".as_bytes()).unwrap();
        assert_eq!(names, vec!["Español", "Zoe"]);
        assert!(parse_voice_names(br#"{"name":"voice"}"#).is_err());
        assert!(parse_voice_names(br#"["bad\nname"]"#).is_err());
        assert_eq!(parse_voice_names(b"[]").unwrap(), Vec::<String>::new());
    }

    #[test]
    fn oversized_speech_is_rejected_without_a_process() {
        let mut speech = Speech::default();
        assert!(speech.speak(&"x".repeat(MAX_SPEECH_BYTES + 1)).is_err());
        assert!(speech.stop().is_ok());
    }

    #[test]
    fn narrates_powers_fractions_and_unary_signs() {
        assert_eq!(
            describe_math("x^2 + x^3 = -1/2"),
            "x squared plus x cubed equals negative 1 divided by 2"
        );
        assert_eq!(
            describe_math("x ^ 12 - 3!"),
            "x to the power of 12 minus 3 factorial"
        );
        assert_eq!(
            describe_math("1.25e-6"),
            "1.25 times ten to the power of negative 6"
        );
        assert_eq!(describe_math("x^2 - 1"), "x squared minus 1");
    }

    #[test]
    fn narrates_functions_and_unicode_math() {
        assert_eq!(
            describe_math("asin(x) + sqrt(4)"),
            "inverse sine of open parenthesis x close parenthesis plus square root of open parenthesis 4 close parenthesis"
        );
        assert_eq!(
            describe_math("π × 2² ± √9"),
            "pi times 2 squared plus or minus square root of 9"
        );
        assert_eq!(
            describe_math("ln(Ans)"),
            "natural logarithm of open parenthesis previous answer close parenthesis"
        );
    }

    #[test]
    fn preserves_error_and_explanation_words() {
        let message = "Cannot divide by zero. Check the cosine and cost assumptions!";
        assert_eq!(describe_math(message), message);
        assert_eq!(
            describe_math("The log contains a root cause."),
            "The log contains a root cause."
        );
        assert_eq!(
            describe_math("Use cos(x), then squarely check your answer."),
            "Use cosine of open parenthesis x close parenthesis, then squarely check your answer."
        );
    }

    #[test]
    fn empty_speech_is_rejected_without_starting_a_process() {
        let mut speech = Speech::default();
        assert!(!speech.is_active());
        assert!(speech.speak("  ").is_err());
        assert!(speech.stop().is_ok());
        assert!(!speech.is_active());
    }
}
