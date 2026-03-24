//! Hardware capability detection for adaptive editor configuration.
//!
//! Uses [`ai_hwaccel`] to probe the system and maps results to
//! editor-relevant quality tiers and feature flags.

#[cfg(feature = "hw")]
use ai_hwaccel::{AcceleratorFamily, AcceleratorProfile, AcceleratorRegistry};

/// Quality tier for rendering/viewport, derived from hardware capabilities.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Default, serde::Serialize, serde::Deserialize,
)]
#[non_exhaustive]
pub enum QualityTier {
    /// Software fallback — CPU only, minimal effects.
    Low,
    /// Integrated or low-end discrete GPU.
    #[default]
    Medium,
    /// Discrete GPU with adequate VRAM (4+ GiB).
    High,
    /// High-end discrete GPU (8+ GiB VRAM).
    Ultra,
}

impl std::fmt::Display for QualityTier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            QualityTier::Low => write!(f, "Low"),
            QualityTier::Medium => write!(f, "Medium"),
            QualityTier::High => write!(f, "High"),
            QualityTier::Ultra => write!(f, "Ultra"),
        }
    }
}

/// Editor-relevant hardware profile.
#[cfg(feature = "hw")]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HardwareProfile {
    /// Recommended quality tier.
    pub quality: QualityTier,
    /// Whether a discrete GPU is available.
    pub has_gpu: bool,
    /// Total GPU/accelerator memory in bytes.
    pub gpu_memory_bytes: u64,
    /// Name of the best available device.
    pub device_name: String,
    /// Number of detected accelerators (excluding CPU).
    pub accelerator_count: usize,
}

#[cfg(feature = "hw")]
impl Default for HardwareProfile {
    fn default() -> Self {
        Self {
            quality: QualityTier::Medium,
            has_gpu: false,
            gpu_memory_bytes: 0,
            device_name: "Unknown".into(),
            accelerator_count: 0,
        }
    }
}

#[cfg(feature = "hw")]
impl HardwareProfile {
    /// Detect hardware and build a profile.
    #[must_use]
    pub fn detect() -> Self {
        let registry = AcceleratorRegistry::detect();
        let profile = Self::from_registry(&registry);
        tracing::debug!(
            quality = %profile.quality,
            device = %profile.device_name,
            gpu = profile.has_gpu,
            "hardware detected"
        );
        profile
    }

    /// Build a profile from an existing registry.
    #[must_use]
    pub fn from_registry(registry: &AcceleratorRegistry) -> Self {
        let has_gpu = registry.has_accelerator();
        let gpu_memory_bytes = registry.total_accelerator_memory();
        let accelerator_count = registry
            .available()
            .iter()
            .filter(|p| !matches!(p.accelerator.family(), AcceleratorFamily::Cpu))
            .count();

        let device_name = registry
            .best_available()
            .map(|p| p.accelerator.to_string())
            .unwrap_or_else(|| "CPU".into());

        let quality = classify_quality(registry.best_available(), gpu_memory_bytes);

        Self {
            quality,
            has_gpu,
            gpu_memory_bytes,
            device_name,
            accelerator_count,
        }
    }

    /// GPU memory in human-readable format.
    #[must_use]
    pub fn gpu_memory_display(&self) -> String {
        if self.gpu_memory_bytes == 0 {
            return "N/A".into();
        }
        let gib = self.gpu_memory_bytes as f64 / (1024.0 * 1024.0 * 1024.0);
        if gib >= 1.0 {
            format!("{gib:.1} GiB")
        } else {
            let mib = self.gpu_memory_bytes as f64 / (1024.0 * 1024.0);
            format!("{mib:.0} MiB")
        }
    }
}

#[cfg(feature = "hw")]
fn classify_quality(best: Option<&AcceleratorProfile>, total_vram: u64) -> QualityTier {
    let Some(profile) = best else {
        return QualityTier::Low;
    };
    if matches!(profile.accelerator.family(), AcceleratorFamily::Cpu) {
        return QualityTier::Low;
    }
    let gib = total_vram as f64 / (1024.0 * 1024.0 * 1024.0);
    if gib < 4.0 {
        QualityTier::Medium
    } else if gib < 8.0 {
        QualityTier::High
    } else {
        QualityTier::Ultra
    }
}

#[cfg(all(test, feature = "hw"))]
mod tests {
    use super::*;

    #[test]
    fn default_profile() {
        let p = HardwareProfile::default();
        assert_eq!(p.quality, QualityTier::Medium);
        assert!(!p.has_gpu);
    }

    #[test]
    fn quality_tier_display() {
        assert_eq!(QualityTier::Low.to_string(), "Low");
        assert_eq!(QualityTier::Ultra.to_string(), "Ultra");
    }

    #[test]
    fn classify_no_gpu() {
        assert_eq!(classify_quality(None, 0), QualityTier::Low);
    }

    #[test]
    fn classify_cpu_only() {
        let cpu = AcceleratorProfile::cpu(16 * 1024 * 1024 * 1024);
        assert_eq!(classify_quality(Some(&cpu), 0), QualityTier::Low);
    }

    #[test]
    fn classify_mid_gpu() {
        let gpu = AcceleratorProfile::cuda(0, 6 * 1024 * 1024 * 1024);
        assert_eq!(
            classify_quality(Some(&gpu), 6 * 1024 * 1024 * 1024),
            QualityTier::High
        );
    }

    #[test]
    fn classify_high_gpu() {
        let gpu = AcceleratorProfile::cuda(0, 8 * 1024 * 1024 * 1024);
        assert_eq!(
            classify_quality(Some(&gpu), 8 * 1024 * 1024 * 1024),
            QualityTier::Ultra
        );
    }

    #[test]
    fn classify_boundary_4gib() {
        // Exactly 4 GiB should be High, not Medium
        let gpu = AcceleratorProfile::cuda(0, 4 * 1024 * 1024 * 1024);
        assert_eq!(
            classify_quality(Some(&gpu), 4 * 1024 * 1024 * 1024),
            QualityTier::High
        );
    }

    #[test]
    fn detect_returns_valid() {
        let p = HardwareProfile::detect();
        assert!(!p.device_name.is_empty());
    }

    #[test]
    fn gpu_memory_display() {
        let mut p = HardwareProfile::default();
        assert_eq!(p.gpu_memory_display(), "N/A");
        p.gpu_memory_bytes = 8 * 1024 * 1024 * 1024;
        assert_eq!(p.gpu_memory_display(), "8.0 GiB");
    }
}
