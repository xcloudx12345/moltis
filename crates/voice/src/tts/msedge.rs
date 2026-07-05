//! MSEdge TTS provider implementation using the `msedge-tts` crate.
//!
//! Wraps the Microsoft Edge Read Aloud API, which provides high-quality
//! neural voices for free without requiring an API key.

use {
    anyhow::{Context, Result},
    async_trait::async_trait,
    bytes::Bytes,
    msedge_tts::{
        tts::SpeechConfig,
        tts::client::SynthesizedAudio,
        tts::client::tokio_runtime::connect_async,
        voice::tokio_runtime::get_voices_list_async,
    },
};

use super::{
    AudioFormat, AudioOutput, SynthesizeRequest, TtsProvider, Voice as ProviderVoice,
};

const DEFAULT_VOICE: &str = "vi-VN-NamMinhNeural";

#[derive(Clone)]
pub struct MSEdgeTts {
    default_voice: String,
}

impl std::fmt::Debug for MSEdgeTts {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MSEdgeTts")
            .field("default_voice", &self.default_voice)
            .finish_non_exhaustive()
    }
}

impl Default for MSEdgeTts {
    fn default() -> Self {
        Self::new(None)
    }
}

impl MSEdgeTts {
    pub fn new(default_voice: Option<String>) -> Self {
        Self {
            default_voice: default_voice.unwrap_or_else(|| DEFAULT_VOICE.into()),
        }
    }

    /// Map our AudioFormat to MSEdge audio format strings.
    fn map_audio_format(&self, format: AudioFormat) -> &'static str {
        match format {
            AudioFormat::Opus | AudioFormat::Webm => "webm-24khz-16bit-mono-opus",
            AudioFormat::Mp3 | AudioFormat::Aac => "audio-24khz-48kbitrate-mono-mp3",
            AudioFormat::Pcm | AudioFormat::Wav => "audio-24khz-48kbitrate-mono-mp3",
        }
    }

    /// Escape XML special characters in text before sending to MSEdge.
    ///
    /// MSEdge wraps synthesis input in SSML, so unescaped `<`, `>`, and `&`
    /// would break the XML parser.
    fn escape_xml(text: &str) -> String {
        text.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
    }
}

#[async_trait]
impl TtsProvider for MSEdgeTts {
    fn id(&self) -> &'static str {
        "msedge"
    }

    fn name(&self) -> &'static str {
        "MSEdge TTS"
    }

    fn is_configured(&self) -> bool {
        true
    }

    fn supports_ssml(&self) -> bool {
        false
    }

    async fn voices(&self) -> Result<Vec<ProviderVoice>> {
        let voices = get_voices_list_async()
            .await
            .context("failed to fetch MSEdge voice list")?;

        Ok(voices
            .into_iter()
            .filter_map(|v| {
                let id = v.short_name.unwrap_or_else(|| v.name.clone());
                let name = v.friendly_name.unwrap_or_else(|| {
                    v.short_name
                        .clone()
                        .unwrap_or_else(|| v.name.clone())
                });
                let description = v
                    .locale
                    .map(|locale| format!("{} - {} ({})", locale, v.gender.unwrap_or_default(), id));
                Some(ProviderVoice {
                    id,
                    name,
                    description,
                    preview_url: None,
                })
            })
            .collect())
    }

    async fn synthesize(&self, request: SynthesizeRequest) -> Result<AudioOutput> {
        let voice_name = request
            .voice_id
            .clone()
            .unwrap_or_else(|| self.default_voice.clone());

        let format_str = self.map_audio_format(request.output_format);
        let rate = request
            .speed
            .map(|s| ((s - 1.0) * 100.0).round() as i32)
            .unwrap_or(0);

        let config = SpeechConfig {
            voice_name,
            audio_format: format_str.to_string(),
            pitch: 0,
            rate,
            volume: 0,
        };

        let escaped_text = Self::escape_xml(&request.text);

        let mut client = connect_async()
            .await
            .context("failed to connect to MSEdge TTS service")?;

        let audio: SynthesizedAudio = client
            .synthesize(&escaped_text, &config)
            .await
            .context("MSEdge TTS synthesis failed")?;

        let output_format = match audio.audio_format.as_str() {
            s if s.starts_with("webm") || s.starts_with("ogg") => AudioFormat::Opus,
            s if s.starts_with("audio") && s.contains("mp3") => AudioFormat::Mp3,
            s if s.starts_with("riff") => AudioFormat::Wav,
            _ => request.output_format,
        };

        Ok(AudioOutput {
            data: Bytes::from(audio.audio_bytes),
            format: output_format,
            duration_ms: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_metadata() {
        let provider = MSEdgeTts::new(None);
        assert_eq!(provider.id(), "msedge");
        assert_eq!(provider.name(), "MSEdge TTS");
        assert!(provider.is_configured());
        assert!(!provider.supports_ssml());
    }

    #[test]
    fn test_default_voice() {
        let provider = MSEdgeTts::new(None);
        assert_eq!(provider.default_voice, "vi-VN-NamMinhNeural");
    }

    #[test]
    fn test_custom_voice() {
        let provider = MSEdgeTts::new(Some("en-US-AriaNeural".into()));
        assert_eq!(provider.default_voice, "en-US-AriaNeural");
    }

    #[test]
    fn test_map_audio_format() {
        let provider = MSEdgeTts::default();
        assert_eq!(
            provider.map_audio_format(AudioFormat::Mp3),
            "audio-24khz-48kbitrate-mono-mp3"
        );
        assert_eq!(
            provider.map_audio_format(AudioFormat::Opus),
            "webm-24khz-16bit-mono-opus"
        );
        assert_eq!(
            provider.map_audio_format(AudioFormat::Webm),
            "webm-24khz-16bit-mono-opus"
        );
        assert_eq!(
            provider.map_audio_format(AudioFormat::Aac),
            "audio-24khz-48kbitrate-mono-mp3"
        );
        assert_eq!(
            provider.map_audio_format(AudioFormat::Pcm),
            "audio-24khz-48kbitrate-mono-mp3"
        );
        assert_eq!(
            provider.map_audio_format(AudioFormat::Wav),
            "audio-24khz-48kbitrate-mono-mp3"
        );
    }

    #[test]
    fn test_xml_escape_noop() {
        assert_eq!(MSEdgeTts::escape_xml("Hello world"), "Hello world");
    }

    #[test]
    fn test_xml_escape_ampersand() {
        let result = MSEdgeTts::escape_xml("Tom & Jerry");
        assert_eq!(result, "Tom &amp; Jerry");
    }

    #[test]
    fn test_xml_escape_less_than() {
        let result = MSEdgeTts::escape_xml("5 < 10");
        assert_eq!(result, "5 &lt; 10");
    }

    #[test]
    fn test_xml_escape_greater_than() {
        let result = MSEdgeTts::escape_xml("10 > 5");
        assert_eq!(result, "10 &gt; 5");
    }

    #[test]
    fn test_xml_escape_combined() {
        let result = MSEdgeTts::escape_xml("a < b && c > d");
        assert_eq!(result, "a &lt; b &amp;&amp; c &gt; d");
    }

    #[test]
    fn test_rate_conversion() {
        // Speed 1.0 -> 0%
        let rate = ((1.0 - 1.0) * 100.0).round() as i32;
        assert_eq!(rate, 0);
        // Speed 1.5 -> +50%
        let rate = ((1.5 - 1.0) * 100.0).round() as i32;
        assert_eq!(rate, 50);
        // Speed 0.8 -> -20%
        let rate = ((0.8 - 1.0) * 100.0).round() as i32;
        assert_eq!(rate, -20);
    }
}
