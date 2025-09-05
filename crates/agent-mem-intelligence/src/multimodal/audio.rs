//! 音频内容处理模块

use super::{MultimodalProcessor, MultimodalContent, ContentType};
use agent_mem_traits::{AgentMemError, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// 音频处理器
#[derive(Debug)]
pub struct AudioProcessor {
    /// 是否启用语音转文本
    pub enable_speech_to_text: bool,
    /// 是否启用音频分析
    pub enable_audio_analysis: bool,
}

impl AudioProcessor {
    /// 创建新的音频处理器
    pub fn new() -> Self {
        Self {
            enable_speech_to_text: true,
            enable_audio_analysis: true,
        }
    }

    /// 配置语音转文本
    pub fn with_speech_to_text(mut self, enable: bool) -> Self {
        self.enable_speech_to_text = enable;
        self
    }

    /// 配置音频分析
    pub fn with_audio_analysis(mut self, enable: bool) -> Self {
        self.enable_audio_analysis = enable;
        self
    }

    /// 执行语音转文本
    async fn speech_to_text(&self, content: &MultimodalContent) -> Result<Option<String>> {
        if !self.enable_speech_to_text {
            return Ok(None);
        }

        // 模拟语音转文本处理
        // 在实际实现中，这里会调用语音识别服务（如 Google Speech-to-Text、Azure Speech 等）
        if content.mime_type.as_ref().map_or(false, |m| m.starts_with("audio/")) {
            let transcribed_text = format!("Transcribed text from audio {}", content.id);
            return Ok(Some(transcribed_text));
        }

        Ok(None)
    }

    /// 分析音频特征
    async fn analyze_audio(&self, content: &MultimodalContent) -> Result<AudioAnalysis> {
        if !self.enable_audio_analysis {
            return Ok(AudioAnalysis::default());
        }

        // 模拟音频分析
        // 在实际实现中，这里会分析音频的各种特征
        Ok(AudioAnalysis {
            duration_seconds: 120.5,
            sample_rate: 44100,
            channels: 2,
            format: "mp3".to_string(),
            volume_level: VolumeLevel::Medium,
            has_speech: true,
            has_music: false,
            dominant_frequency: 440.0,
            confidence: 0.85,
        })
    }
}

impl Default for AudioProcessor {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl MultimodalProcessor for AudioProcessor {
    async fn process(&self, content: &mut MultimodalContent) -> Result<()> {
        // 验证内容类型
        if content.content_type != ContentType::Audio {
            return Err(AgentMemError::ProcessingError(
                "AudioProcessor can only process audio content".to_string(),
            ));
        }

        // 执行语音转文本
        if let Ok(Some(transcribed_text)) = self.speech_to_text(content).await {
            content.set_extracted_text(transcribed_text);
        }

        // 分析音频特征
        if let Ok(analysis) = self.analyze_audio(content).await {
            let analysis_json = serde_json::to_value(analysis)
                .map_err(|e| AgentMemError::ProcessingError(format!("Failed to serialize audio analysis: {}", e)))?;
            content.set_metadata("audio_analysis".to_string(), analysis_json);
        }

        Ok(())
    }

    fn supported_types(&self) -> Vec<ContentType> {
        vec![ContentType::Audio]
    }

    async fn extract_text(&self, content: &MultimodalContent) -> Result<Option<String>> {
        self.speech_to_text(content).await
    }

    async fn generate_summary(&self, content: &MultimodalContent) -> Result<Option<String>> {
        let mut summary_parts = Vec::new();

        summary_parts.push(format!("Audio: {}", content.id));

        if let Some(text) = &content.extracted_text {
            summary_parts.push(format!("Transcription: {}", text));
        }

        if let Some(analysis_value) = content.get_metadata("audio_analysis") {
            if let Ok(analysis) = serde_json::from_value::<AudioAnalysis>(analysis_value.clone()) {
                summary_parts.push(format!(
                    "Duration: {:.1}s, Format: {}, Volume: {:?}",
                    analysis.duration_seconds, analysis.format, analysis.volume_level
                ));
            }
        }

        Ok(Some(summary_parts.join(". ")))
    }
}

/// 音频分析结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioAnalysis {
    /// 持续时间（秒）
    pub duration_seconds: f64,
    /// 采样率
    pub sample_rate: u32,
    /// 声道数
    pub channels: u32,
    /// 音频格式
    pub format: String,
    /// 音量级别
    pub volume_level: VolumeLevel,
    /// 是否包含语音
    pub has_speech: bool,
    /// 是否包含音乐
    pub has_music: bool,
    /// 主要频率
    pub dominant_frequency: f64,
    /// 置信度
    pub confidence: f32,
}

impl Default for AudioAnalysis {
    fn default() -> Self {
        Self {
            duration_seconds: 0.0,
            sample_rate: 44100,
            channels: 1,
            format: "unknown".to_string(),
            volume_level: VolumeLevel::Medium,
            has_speech: false,
            has_music: false,
            dominant_frequency: 0.0,
            confidence: 0.0,
        }
    }
}

/// 音量级别
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum VolumeLevel {
    Silent,
    Low,
    Medium,
    High,
    VeryHigh,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_audio_processor_creation() {
        let processor = AudioProcessor::new();
        assert!(processor.enable_speech_to_text);
        assert!(processor.enable_audio_analysis);
    }

    #[tokio::test]
    async fn test_audio_processor_configuration() {
        let processor = AudioProcessor::new()
            .with_speech_to_text(false)
            .with_audio_analysis(true);

        assert!(!processor.enable_speech_to_text);
        assert!(processor.enable_audio_analysis);
    }

    #[tokio::test]
    async fn test_audio_processing() {
        let processor = AudioProcessor::new();
        let mut content = MultimodalContent::from_data(
            "test-audio".to_string(),
            vec![1, 2, 3, 4, 5],
            "audio/mp3".to_string(),
        );

        let result = processor.process(&mut content).await;
        assert!(result.is_ok());
        assert!(content.extracted_text.is_some());
        assert!(content.get_metadata("audio_analysis").is_some());
    }

    #[test]
    fn test_audio_analysis() {
        let analysis = AudioAnalysis {
            duration_seconds: 120.5,
            sample_rate: 44100,
            channels: 2,
            format: "mp3".to_string(),
            volume_level: VolumeLevel::Medium,
            has_speech: true,
            has_music: false,
            dominant_frequency: 440.0,
            confidence: 0.85,
        };

        assert_eq!(analysis.duration_seconds, 120.5);
        assert_eq!(analysis.sample_rate, 44100);
        assert_eq!(analysis.volume_level, VolumeLevel::Medium);
        assert!(analysis.has_speech);
        assert!(!analysis.has_music);
    }
}
