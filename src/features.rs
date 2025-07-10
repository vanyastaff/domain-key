//! Feature detection and runtime performance information for domain-key
//!
//! This module provides utilities for detecting enabled features at runtime,
//! getting performance information, and understanding the optimization profile
//! of the current build.

use core::fmt;

#[cfg(not(feature = "std"))]
use alloc::string::String;
#[cfg(not(feature = "std"))]
use alloc::vec::Vec;
#[cfg(not(feature = "std"))]
use alloc::format;

// ============================================================================
// PERFORMANCE INFORMATION
// ============================================================================

/// Runtime performance information
///
/// This structure provides comprehensive information about the current
/// performance configuration, including active features, hash algorithms,
/// and estimated performance improvements.
#[derive(Debug, Clone)]
pub struct PerformanceInfo {
    /// Active hash algorithm
    pub hash_algorithm: &'static str,
    /// Whether standard library features are enabled
    pub has_std: bool,
    /// Whether serialization support is enabled
    pub has_serde: bool,
    /// Hash algorithm category for performance characteristics
    pub hash_category: HashCategory,
    /// Estimated performance improvement over baseline
    pub estimated_improvement: f32,
    /// Memory usage characteristics
    pub memory_profile: MemoryProfile,
    /// Build configuration details
    pub build_info: BuildInfo,
}

impl fmt::Display for PerformanceInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "domain-key Performance Configuration:")?;
        writeln!(
            f,
            "  Hash Algorithm: {} ({})",
            self.hash_algorithm, self.hash_category
        )?;
        writeln!(f, "  Standard Library: {}", self.has_std)?;
        writeln!(f, "  Serialization: {}", self.has_serde)?;
        writeln!(
            f,
            "  Performance Multiplier: {:.1}x baseline",
            self.estimated_improvement
        )?;
        writeln!(f, "  Memory Profile: {}", self.memory_profile)?;
        writeln!(f, "  Build: {}", self.build_info)?;
        Ok(())
    }
}

/// Hash algorithm categories with performance characteristics
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HashCategory {
    /// Ultra-fast hashing for maximum performance (GxHash)
    UltraFast,
    /// Balanced performance with DoS protection (AHash)
    Secure,
    /// Cryptographically secure hashing (Blake3)
    Cryptographic,
    /// Standard library default hasher
    Standard,
    /// Simple hash for no_std environments
    Simple,
}

impl fmt::Display for HashCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UltraFast => write!(f, "ultra-fast"),
            Self::Secure => write!(f, "secure"),
            Self::Cryptographic => write!(f, "cryptographic"),
            Self::Standard => write!(f, "standard"),
            Self::Simple => write!(f, "simple"),
        }
    }
}

/// Memory usage profile information
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MemoryProfile {
    /// Whether stack allocation optimizations are available
    pub stack_optimized: bool,
    /// Whether length caching is enabled
    pub length_cached: bool,
    /// Whether hash caching is enabled
    pub hash_cached: bool,
    /// Estimated memory overhead per key (in bytes)
    pub overhead_per_key: usize,
}

impl fmt::Display for MemoryProfile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "stack:{}, length_cache:{}, hash_cache:{}, overhead:{}B",
            self.stack_optimized, self.length_cached, self.hash_cached, self.overhead_per_key
        )
    }
}

/// Build configuration information
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BuildInfo {
    /// Whether built in release mode
    pub is_release: bool,
    /// Whether LTO is likely enabled
    pub has_lto: bool,
    /// Target architecture category
    pub arch_category: ArchCategory,
}

impl fmt::Display for BuildInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "release:{}, lto:{}, arch:{}",
            self.is_release, self.has_lto, self.arch_category
        )
    }
}

/// Target architecture categories
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArchCategory {
    /// x86_64 with modern features
    X86_64Modern,
    /// x86_64 compatible
    X86_64,
    /// ARM64/AArch64
    ARM64,
    /// ARM 32-bit
    ARM32,
    /// Other/unknown architecture
    Other,
}

impl fmt::Display for ArchCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::X86_64Modern => write!(f, "x86_64-modern"),
            Self::X86_64 => write!(f, "x86_64"),
            Self::ARM64 => write!(f, "arm64"),
            Self::ARM32 => write!(f, "arm32"),
            Self::Other => write!(f, "other"),
        }
    }
}

// ============================================================================
// FEATURE DETECTION FUNCTIONS
// ============================================================================

/// Returns true if standard library support is enabled
pub const fn has_std() -> bool {
    cfg!(feature = "std")
}

/// Returns true if serialization support is enabled
pub const fn has_serde() -> bool {
    cfg!(feature = "serde")
}

/// Returns true if the fast hash algorithm is enabled
pub const fn has_fast_hash() -> bool {
    cfg!(feature = "fast")
}

/// Returns true if the secure hash algorithm is enabled
pub const fn has_secure_hash() -> bool {
    cfg!(feature = "secure")
}

/// Returns true if the cryptographic hash algorithm is enabled
pub const fn has_crypto_hash() -> bool {
    cfg!(feature = "crypto")
}

/// Returns the active hash algorithm name
pub const fn hash_algorithm() -> &'static str {
    // Priority-based selection to handle multiple features during testing
    #[cfg(feature = "fast")]
    {
        #[cfg(any(
            all(target_arch = "x86_64", target_feature = "aes"),
            all(target_arch = "aarch64", target_feature = "aes")
        ))]
        return "GxHash";

        #[cfg(not(any(
            all(target_arch = "x86_64", target_feature = "aes"),
            all(target_arch = "aarch64", target_feature = "aes")
        )))]
        return "AHash (GxHash fallback)";
    }

    #[cfg(all(feature = "secure", not(feature = "fast")))]
    return "AHash";

    #[cfg(all(feature = "crypto", not(any(feature = "fast", feature = "secure"))))]
    return "Blake3";

    #[cfg(not(any(feature = "fast", feature = "secure", feature = "crypto")))]
    {
        #[cfg(feature = "std")]
        return "DefaultHasher";

        #[cfg(not(feature = "std"))]
        return "FNV-1a";
    }
}

/// Returns the hash algorithm category
pub const fn hash_category() -> HashCategory {
    #[cfg(feature = "fast")]
    {
        #[cfg(any(
            all(target_arch = "x86_64", target_feature = "aes"),
            all(target_arch = "aarch64", target_feature = "aes")
        ))]
        return HashCategory::UltraFast;

        #[cfg(not(any(
            all(target_arch = "x86_64", target_feature = "aes"),
            all(target_arch = "aarch64", target_feature = "aes")
        )))]
        return HashCategory::Secure; // Falls back to AHash
    }

    #[cfg(all(feature = "secure", not(feature = "fast")))]
    return HashCategory::Secure;

    #[cfg(all(feature = "crypto", not(any(feature = "fast", feature = "secure"))))]
    return HashCategory::Cryptographic;

    #[cfg(not(any(feature = "fast", feature = "secure", feature = "crypto")))]
    {
        #[cfg(feature = "std")]
        return HashCategory::Standard;

        #[cfg(not(feature = "std"))]
        return HashCategory::Simple;
    }
}

/// Estimate performance improvement over baseline
const fn estimate_performance_improvement() -> f32 {
    let base_multiplier = 1.0;

    // Hash algorithm impact
    let hash_multiplier = {
        #[cfg(feature = "fast")]
        {
            #[cfg(any(
                all(target_arch = "x86_64", target_feature = "aes"),
                all(target_arch = "aarch64", target_feature = "aes")
            ))]
            { 1.4 }
            #[cfg(not(any(
                all(target_arch = "x86_64", target_feature = "aes"),
                all(target_arch = "aarch64", target_feature = "aes")
            )))]
            { 1.2 }
        }
        #[cfg(all(feature = "secure", not(feature = "fast")))]
        { 1.0 }
        #[cfg(all(feature = "crypto", not(any(feature = "fast", feature = "secure"))))]
        { 0.8 }
        #[cfg(not(any(feature = "fast", feature = "secure", feature = "crypto")))]
        { 1.0 }
    };

    // Standard library optimizations
    let std_multiplier = {
        #[cfg(feature = "std")]
        { 1.1 }
        #[cfg(not(feature = "std"))]
        { 1.0 }
    };

    base_multiplier * hash_multiplier * std_multiplier
}

/// Get memory profile information
const fn memory_profile() -> MemoryProfile {
    MemoryProfile {
        stack_optimized: true, // SmartString provides stack optimization
        length_cached: true,   // We always cache length
        hash_cached: true,     // We always cache hash
        overhead_per_key: core::mem::size_of::<u64>() + core::mem::size_of::<u32>(), // hash + length
    }
}

/// Detect build configuration
const fn build_info() -> BuildInfo {
    BuildInfo {
        is_release: cfg!(not(debug_assertions)),
        has_lto: cfg!(not(debug_assertions)), // Assume LTO in release builds
        arch_category: detect_arch_category(),
    }
}

/// Detect target architecture category
const fn detect_arch_category() -> ArchCategory {
    #[cfg(all(target_arch = "x86_64", target_feature = "aes"))]
    {
        ArchCategory::X86_64Modern
    }
    #[cfg(all(target_arch = "x86_64", not(target_feature = "aes")))]
    {
        ArchCategory::X86_64
    }
    #[cfg(target_arch = "aarch64")]
    {
        ArchCategory::ARM64
    }
    #[cfg(target_arch = "arm")]
    {
        ArchCategory::ARM32
    }
    #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64", target_arch = "arm")))]
    {
        ArchCategory::Other
    }
}

/// Get comprehensive runtime performance information
pub const fn performance_info() -> PerformanceInfo {
    PerformanceInfo {
        hash_algorithm: hash_algorithm(),
        has_std: has_std(),
        has_serde: has_serde(),
        hash_category: hash_category(),
        estimated_improvement: estimate_performance_improvement(),
        memory_profile: memory_profile(),
        build_info: build_info(),
    }
}

// ============================================================================
// OPTIMIZATION DETECTION
// ============================================================================

/// Check if length caching optimizations are available
///
/// Length caching provides O(1) length access instead of O(n) string traversal.
pub const fn has_length_caching() -> bool {
    true // Always enabled in this implementation
}

/// Check if hash caching optimizations are available
///
/// Hash caching provides O(1) hash access for hash-based collections.
pub const fn has_hash_caching() -> bool {
    true // Always enabled in this implementation
}

/// Check if stack allocation optimizations are available
///
/// Stack allocation reduces heap allocations for short keys.
pub const fn has_stack_optimization() -> bool {
    true // SmartString provides this automatically
}

/// Check if SIMD optimizations might be available
///
/// This is a compile-time check for SIMD features that might
/// be used by the hash algorithms.
pub const fn has_simd_support() -> bool {
    #[cfg(any(
        target_feature = "sse2",
        target_feature = "neon",
        target_feature = "aes"
    ))]
    return true;

    #[cfg(not(any(
        target_feature = "sse2",
        target_feature = "neon",
        target_feature = "aes"
    )))]
    return false;
}

/// Check if the current configuration is optimized for security
pub const fn is_security_optimized() -> bool {
    has_secure_hash() || has_crypto_hash()
}

/// Check if the current configuration is optimized for performance
pub const fn is_performance_optimized() -> bool {
    has_fast_hash() && has_std()
}

/// Check if the current configuration is balanced
pub const fn is_balanced_configuration() -> bool {
    !is_performance_optimized() && !is_security_optimized()
}

// ============================================================================
// BENCHMARKING UTILITIES
// ============================================================================

/// Performance benchmark results
#[derive(Debug, Clone)]
pub struct BenchmarkResults {
    /// Key creation time (nanoseconds per operation)
    pub creation_ns: u64,
    /// Hash computation time (nanoseconds per operation)
    pub hash_ns: u64,
    /// Length access time (nanoseconds per operation)
    pub length_ns: u64,
    /// Comparison time (nanoseconds per operation)
    pub comparison_ns: u64,
    /// Memory usage per key (bytes)
    pub memory_bytes: usize,
}

impl fmt::Display for BenchmarkResults {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Performance Benchmark Results:")?;
        writeln!(f, "  Key Creation: {} ns/op", self.creation_ns)?;
        writeln!(f, "  Hash Access: {} ns/op", self.hash_ns)?;
        writeln!(f, "  Length Access: {} ns/op", self.length_ns)?;
        writeln!(f, "  Comparison: {} ns/op", self.comparison_ns)?;
        writeln!(f, "  Memory Usage: {} bytes/key", self.memory_bytes)?;
        Ok(())
    }
}

/// Estimate benchmark results based on current configuration
///
/// This provides rough estimates of performance characteristics
/// based on the enabled features and target architecture.
pub fn estimated_benchmark_results() -> BenchmarkResults {
    let info = performance_info();
    let base_creation_ns = 100;
    let base_hash_ns = 10;
    let base_length_ns = 5;
    let base_comparison_ns = 15;

    BenchmarkResults {
        creation_ns: (base_creation_ns as f32 / info.estimated_improvement) as u64,
        hash_ns: match info.hash_category {
            HashCategory::UltraFast => base_hash_ns / 2,
            HashCategory::Secure => base_hash_ns,
            HashCategory::Cryptographic => base_hash_ns * 2,
            HashCategory::Standard => base_hash_ns,
            HashCategory::Simple => base_hash_ns,
        },
        length_ns: if has_length_caching() {
            1
        } else {
            base_length_ns
        },
        comparison_ns: if has_hash_caching() {
            base_comparison_ns / 3
        } else {
            base_comparison_ns
        },
        memory_bytes: info.memory_profile.overhead_per_key + 24, // Base SmartString size
    }
}

// ============================================================================
// FEATURE RECOMMENDATIONS
// ============================================================================

/// Recommendations for feature selection based on use case
#[derive(Debug, Clone)]
pub struct FeatureRecommendations {
    /// Recommended features for maximum performance
    pub performance: &'static [&'static str],
    /// Recommended features for security-sensitive applications
    pub security: &'static [&'static str],
    /// Recommended features for balanced use cases
    pub balanced: &'static [&'static str],
    /// Recommended features for minimal dependencies
    pub minimal: &'static [&'static str],
}

/// Get feature recommendations for different use cases
pub fn feature_recommendations() -> FeatureRecommendations {
    FeatureRecommendations {
        performance: &["fast", "std", "serde"],
        security: &["secure", "std", "serde"],
        balanced: &["std", "serde"],
        minimal: &[],
    }
}

/// Get recommendations based on current configuration
pub fn analyze_current_configuration() -> ConfigurationAnalysis {
    let info = performance_info();

    ConfigurationAnalysis {
        overall_score: calculate_overall_score(&info),
        strengths: analyze_strengths(&info),
        weaknesses: analyze_weaknesses(&info),
        suggestions: generate_suggestions(&info),
    }
}

/// Analysis of the current configuration
#[derive(Debug, Clone)]
pub struct ConfigurationAnalysis {
    /// Overall configuration score (0-100)
    pub overall_score: u8,
    /// Configuration strengths
    pub strengths: Vec<&'static str>,
    /// Configuration weaknesses
    pub weaknesses: Vec<&'static str>,
    /// Improvement suggestions
    pub suggestions: Vec<&'static str>,
}

impl fmt::Display for ConfigurationAnalysis {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "Configuration Analysis (Score: {}/100):",
            self.overall_score
        )?;

        if !self.strengths.is_empty() {
            writeln!(f, "\n✅ Strengths:")?;
            for strength in &self.strengths {
                writeln!(f, "  • {}", strength)?;
            }
        }

        if !self.weaknesses.is_empty() {
            writeln!(f, "\n⚠️  Weaknesses:")?;
            for weakness in &self.weaknesses {
                writeln!(f, "  • {}", weakness)?;
            }
        }

        if !self.suggestions.is_empty() {
            writeln!(f, "\n💡 Suggestions:")?;
            for suggestion in &self.suggestions {
                writeln!(f, "  • {}", suggestion)?;
            }
        }

        Ok(())
    }
}

fn calculate_overall_score(info: &PerformanceInfo) -> u8 {
    let mut score = 50; // Base score

    // Hash algorithm scoring
    score += match info.hash_category {
        HashCategory::UltraFast => 25,
        HashCategory::Secure => 20,
        HashCategory::Cryptographic => 15,
        HashCategory::Standard => 10,
        HashCategory::Simple => 5,
    };

    // Feature scoring
    if info.has_std {
        score += 15;
    }
    if info.has_serde {
        score += 10;
    }

    score.min(100)
}

fn analyze_strengths(info: &PerformanceInfo) -> Vec<&'static str> {
    let mut strengths = Vec::new();

    match info.hash_category {
        HashCategory::UltraFast => strengths.push("Ultra-fast hashing with GxHash"),
        HashCategory::Secure => strengths.push("DoS-resistant hashing with AHash"),
        HashCategory::Cryptographic => strengths.push("Cryptographically secure hashing"),
        _ => {}
    }

    if info.has_std {
        strengths.push("Full standard library optimizations");
    }

    if info.has_serde {
        strengths.push("Serialization support for data exchange");
    }

    if info.memory_profile.hash_cached {
        strengths.push("O(1) hash access with caching");
    }

    if info.memory_profile.length_cached {
        strengths.push("O(1) length access with caching");
    }

    strengths
}

fn analyze_weaknesses(info: &PerformanceInfo) -> Vec<&'static str> {
    let mut weaknesses = Vec::new();

    if !info.has_std {
        weaknesses.push("Missing standard library optimizations");
    }

    if !info.has_serde {
        weaknesses.push("No serialization support");
    }

    match info.hash_category {
        HashCategory::Simple => weaknesses.push("Basic hash algorithm may have poor distribution"),
        HashCategory::Standard => {
            weaknesses.push("Standard hasher may be vulnerable to DoS attacks")
        }
        _ => {}
    }

    if info.estimated_improvement < 1.0 {
        weaknesses.push("Performance below baseline due to security overhead");
    }

    weaknesses
}

fn generate_suggestions(info: &PerformanceInfo) -> Vec<&'static str> {
    let mut suggestions = Vec::new();

    if !info.has_std {
        suggestions.push("Enable 'std' feature for better performance");
    }

    if !info.has_serde {
        suggestions.push("Enable 'serde' feature for serialization support");
    }

    match info.hash_category {
        HashCategory::Standard | HashCategory::Simple => {
            suggestions.push("Consider 'fast' feature for better performance");
            suggestions.push("Consider 'secure' feature for DoS protection");
        }
        _ => {}
    }

    if !has_simd_support() {
        suggestions.push("Consider compiling with target-cpu=native for SIMD optimizations");
    }

    suggestions
}

// ============================================================================
// DIAGNOSTICS
// ============================================================================

/// Print comprehensive diagnostic information
#[cfg(feature = "std")]
pub fn print_diagnostics() {
    let info = performance_info();
    let analysis = analyze_current_configuration();
    let benchmarks = estimated_benchmark_results();

    println!("{}", info);
    println!();
    println!("{}", analysis);
    println!();
    println!("{}", benchmarks);
}

/// Get a summary of the current configuration
pub fn configuration_summary() -> String {
    let info = performance_info();
    format!(
        "domain-key {} | {} hash | {:.1}x performance | {}",
        if info.has_std { "std" } else { "no_std" },
        info.hash_algorithm,
        info.estimated_improvement,
        info.build_info
    )
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature_detection() {
        // These tests verify that feature detection works
        println!("Has std: {}", has_std());
        println!("Has serde: {}", has_serde());
        println!("Hash algorithm: {}", hash_algorithm());
        println!("Hash category: {}", hash_category());

        // Should not panic
        let info = performance_info();
        assert!(!info.hash_algorithm.is_empty());
    }

    #[test]
    fn test_performance_info_display() {
        let info = performance_info();
        let display = format!("{}", info);
        assert!(display.contains("domain-key Performance Configuration"));
        assert!(display.contains("Hash Algorithm"));
    }

    #[test]
    fn test_memory_profile() {
        let profile = memory_profile();
        assert!(profile.stack_optimized);
        assert!(profile.length_cached);
        assert!(profile.hash_cached);
        assert!(profile.overhead_per_key > 0);
    }

    #[test]
    fn test_benchmark_estimates() {
        let results = estimated_benchmark_results();
        assert!(results.creation_ns > 0);
        assert!(results.hash_ns > 0);
        assert!(results.length_ns > 0);
        assert!(results.comparison_ns > 0);
        assert!(results.memory_bytes > 0);
    }

    #[test]
    fn test_feature_recommendations() {
        let recs = feature_recommendations();
        assert!(!recs.performance.is_empty());
        assert!(!recs.security.is_empty());
        assert!(!recs.balanced.is_empty());
        // Minimal can be empty
    }

    #[test]
    fn test_configuration_analysis() {
        let analysis = analyze_current_configuration();
        assert!(analysis.overall_score <= 100);

        let display = format!("{}", analysis);
        assert!(display.contains("Configuration Analysis"));
    }

    #[test]
    fn test_diagnostics() {
        // Should not panic
        let summary = configuration_summary();
        assert!(summary.contains("domain-key"));
        assert!(summary.contains("hash"));

        // This would print in a real test run
        // print_diagnostics();
    }

    #[test]
    fn test_optimization_detection() {
        assert!(has_length_caching());
        assert!(has_hash_caching());
        assert!(has_stack_optimization());

        // These depend on compile-time features
        println!("SIMD support: {}", has_simd_support());
        println!("Security optimized: {}", is_security_optimized());
        println!("Performance optimized: {}", is_performance_optimized());
        println!("Balanced: {}", is_balanced_configuration());
    }

    #[test]
    fn test_arch_detection() {
        let arch = detect_arch_category();
        println!("Detected architecture: {}", arch);
        // Should not panic and should return valid category
    }

    #[test]
    fn test_hash_categories() {
        let category = hash_category();
        println!("Hash category: {}", category);

        // Test display
        assert!(!format!("{}", category).is_empty());
    }
}
