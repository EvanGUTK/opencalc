use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum CalculatorLayout {
    #[default]
    Familiar,
    Accessible,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum ColorTheme {
    #[default]
    Light,
    Warm,
    Dark,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum ScanMode {
    #[default]
    Off,
    Automatic,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub(crate) struct AccessibilitySettings {
    pub layout: CalculatorLayout,
    pub theme: ColorTheme,
    pub target_size: f32,
    pub key_spacing: f32,
    pub left_handed: bool,
    pub word_labels: bool,
    pub large_controls: bool,
    pub font_scale: f32,
    pub line_spacing: f32,
    pub focus_mode: bool,
    pub reduced_motion: bool,
    pub reading_guide: bool,
    pub show_help_text: bool,
    pub show_scores: bool,
    pub one_step: bool,
    pub speak_keys: bool,
    pub speak_focus: bool,
    pub speech_rate: i32,
    pub speech_volume: u32,
    pub speech_voice: String,
    pub captions: bool,
    pub dwell_ms: u64,
    pub guard_ms: u64,
    pub scan_mode: ScanMode,
    pub scan_interval_ms: u64,
    pub graph_sound: bool,
    pub decimal_comma: bool,
}
impl Default for AccessibilitySettings {
    fn default() -> Self {
        Self {
            layout: CalculatorLayout::Familiar,
            theme: ColorTheme::Light,
            target_size: 48.0,
            key_spacing: 8.0,
            left_handed: false,
            word_labels: false,
            large_controls: false,
            font_scale: 1.0,
            line_spacing: 1.0,
            focus_mode: false,
            reduced_motion: true,
            reading_guide: false,
            show_help_text: true,
            show_scores: true,
            one_step: true,
            speak_keys: false,
            speak_focus: false,
            speech_rate: 0,
            speech_volume: 80,
            speech_voice: String::new(),
            captions: true,
            dwell_ms: 0,
            guard_ms: 0,
            scan_mode: ScanMode::Off,
            scan_interval_ms: 1500,
            graph_sound: false,
            decimal_comma: false,
        }
    }
}
impl AccessibilitySettings {
    pub fn sanitize(&mut self) {
        let defaults = Self::default();
        if !self.target_size.is_finite() {
            self.target_size = defaults.target_size;
        }
        if !self.key_spacing.is_finite() {
            self.key_spacing = defaults.key_spacing;
        }
        if !self.font_scale.is_finite() {
            self.font_scale = defaults.font_scale;
        }
        if !self.line_spacing.is_finite() {
            self.line_spacing = defaults.line_spacing;
        }
        self.target_size = self.target_size.clamp(40.0, 80.0);
        self.key_spacing = self.key_spacing.clamp(4.0, 24.0);
        self.font_scale = self.font_scale.clamp(0.9, 1.8);
        self.line_spacing = self.line_spacing.clamp(1.0, 2.0);
        self.speech_rate = self.speech_rate.clamp(-5, 5);
        self.speech_volume = self.speech_volume.min(100);
        self.dwell_ms = if self.dwell_ms == 0 {
            0
        } else {
            self.dwell_ms.clamp(600, 4000)
        };
        self.guard_ms = self.guard_ms.min(1500);
        self.scan_interval_ms = self.scan_interval_ms.clamp(600, 5000);
        self.speech_voice = self.speech_voice.chars().take(128).collect();
    }
}

#[derive(Default)]
pub(crate) struct AccessibilityRuntime {
    pub latest_announcement: String,
    pub voices: Vec<String>,
    pub voice_notice: String,
    pub show_shortcuts: bool,
    pub show_glossary: bool,
    pub glossary_search: String,
    pub focus_navigation: bool,
    pub guide_y: f32,
    pub workspace_focus: Option<eframe::egui::Id>,
    pub overlay_return_focus: Option<eframe::egui::Id>,
    pub overlay_was_open: bool,
    pub overlay_had_settings: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn invalid_saved_values_restore_finite_usable_preferences() {
        let mut settings = AccessibilitySettings {
            target_size: f32::NAN,
            key_spacing: f32::INFINITY,
            font_scale: f32::NEG_INFINITY,
            line_spacing: f32::NAN,
            dwell_ms: 1,
            scan_interval_ms: u64::MAX,
            speech_volume: 999,
            speech_rate: -99,
            ..Default::default()
        };
        settings.sanitize();
        assert_eq!(settings.target_size, 48.0);
        assert_eq!(settings.key_spacing, 8.0);
        assert_eq!(settings.font_scale, 1.0);
        assert_eq!(settings.line_spacing, 1.0);
        assert_eq!(settings.dwell_ms, 600);
        assert_eq!(settings.scan_interval_ms, 5000);
        assert_eq!(settings.speech_volume, 100);
        assert_eq!(settings.speech_rate, -5);
    }
    #[test]
    fn older_saved_settings_gain_access_options_without_losing_grade_or_angles() {
        let settings: crate::Settings =
            serde_json::from_str(r#"{"level":1,"degrees":false,"scale":1.25}"#).unwrap();
        assert_eq!(settings.level, 1);
        assert!(!settings.degrees);
        assert_eq!(settings.scale, 1.25);
        assert_eq!(settings.access.layout, CalculatorLayout::Familiar);
        assert!(settings.access.captions);
        assert_eq!(settings.access.scan_mode, ScanMode::Off);
    }
}
