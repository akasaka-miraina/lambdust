//! CI/CD integration for crash reporting and artifact management

use std::collections::HashMap;
use std::fmt;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::debug::{CrashInfo, CrashPattern, FixSuggestion, TestSummary};

/// Comprehensive crash report for CI systems
#[derive(Debug)]
pub struct CrashReport {
    pub build_info: BuildInfo,
    pub environment: EnvironmentInfo,
    pub crashes: Vec<CrashInfo>,
    pub patterns: Vec<CrashPattern>,
    pub suggestions: Vec<FixSuggestion>,
    pub test_summary: Option<TestSummary>,
    pub artifacts: Vec<ArtifactInfo>,
}

#[derive(Debug, Clone)]
pub struct BuildInfo {
    pub commit_hash: String,
    pub branch: String,
    pub build_number: String,
    pub rust_version: String,
    pub compilation_flags: Vec<String>,
    pub timestamp: u64,
}

#[derive(Debug, Clone)]
pub struct EnvironmentInfo {
    pub ci_provider: String,
    pub runner_os: String,
    pub runner_arch: String,
    pub environment_variables: HashMap<String, String>,
    pub system_info: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct ArtifactInfo {
    pub name: String,
    pub path: String,
    pub size_bytes: u64,
    pub artifact_type: ArtifactType,
    pub description: String,
}

#[derive(Debug, Clone)]
pub enum ArtifactType {
    CrashLog,
    CoreDump,
    TestOutput,
    MemoryReport,
    PlatformComparison,
    DebugSymbols,
    Binary,
}

/// CI reporter for uploading crash information and artifacts
pub struct CiReporter {
    output_dir: PathBuf,
    enable_artifact_compression: bool,
    max_artifact_size: u64,
}

impl Default for CiReporter {
    fn default() -> Self {
        Self {
            output_dir: PathBuf::from("ci_artifacts"),
            enable_artifact_compression: true,
            max_artifact_size: 100 * 1024 * 1024, // 100MB
        }
    }
}

impl CiReporter {
    pub fn new(output_dir: PathBuf) -> Self {
        Self {
            output_dir,
            enable_artifact_compression: true,
            max_artifact_size: 100 * 1024 * 1024,
        }
    }

    pub fn with_compression(mut self, enable: bool) -> Self {
        self.enable_artifact_compression = enable;
        self
    }

    pub fn with_max_artifact_size(mut self, size: u64) -> Self {
        self.max_artifact_size = size;
        self
    }

    /// Generate a comprehensive crash report
    pub fn generate_crash_report(
        &self,
        crashes: Vec<CrashInfo>,
        patterns: Vec<CrashPattern>,
        suggestions: Vec<FixSuggestion>,
        test_summary: Option<TestSummary>,
    ) -> Result<CrashReport, Box<dyn std::error::Error>> {
        
        let build_info = self.collect_build_info()?;
        let environment = self.collect_environment_info()?;
        let artifacts = self.collect_artifacts()?;

        Ok(CrashReport {
            build_info,
            environment,
            crashes,
            patterns,
            suggestions,
            test_summary,
            artifacts,
        })
    }

    /// Write crash report to files and prepare for CI upload
    pub fn write_crash_report(&self, report: &CrashReport) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
        // Ensure output directory exists
        fs::create_dir_all(&self.output_dir)?;

        let mut generated_files = Vec::new();

        // Write main crash report
        let report_path = self.output_dir.join("crash_report.md");
        let mut report_file = File::create(&report_path)?;
        write!(report_file, "{}", self.format_crash_report_markdown(report)?)?;
        generated_files.push(report_path);

        // Write JSON version for machine processing
        let json_path = self.output_dir.join("crash_report.json");
        let json_content = self.format_crash_report_json(report)?;
        fs::write(&json_path, json_content)?;
        generated_files.push(json_path);

        // Write individual crash logs
        for (i, crash) in report.crashes.iter().enumerate() {
            let crash_path = self.output_dir.join(format!("crash_{:04}.log", i));
            fs::write(&crash_path, format!("{}", crash))?;
            generated_files.push(crash_path);
        }

        // Write pattern analysis
        let patterns_path = self.output_dir.join("crash_patterns.md");
        let mut patterns_file = File::create(&patterns_path)?;
        self.write_patterns_analysis(&mut patterns_file, &report.patterns, &report.suggestions)?;
        generated_files.push(patterns_path);

        // Write test summary if available
        if let Some(ref summary) = report.test_summary {
            let summary_path = self.output_dir.join("test_summary.md");
            fs::write(&summary_path, format!("{}", summary))?;
            generated_files.push(summary_path);
        }

        // Write GitHub Actions workflow annotations
        let annotations_path = self.output_dir.join("github_annotations.txt");
        self.write_github_annotations(&annotations_path, report)?;
        generated_files.push(annotations_path);

        // Copy and compress artifacts
        for artifact in &report.artifacts {
            if let Ok(copied_path) = self.copy_artifact(artifact) {
                generated_files.push(copied_path);
            }
        }

        Ok(generated_files)
    }

    fn collect_build_info(&self) -> Result<BuildInfo, Box<dyn std::error::Error>> {
        let commit_hash = self.get_git_commit_hash().unwrap_or_else(|_| "unknown".to_string());
        let branch = self.get_git_branch().unwrap_or_else(|_| "unknown".to_string());
        let build_number = std::env::var("GITHUB_RUN_NUMBER").unwrap_or_else(|_| "unknown".to_string());
        let rust_version = self.get_rust_version().unwrap_or_else(|_| "unknown".to_string());

        Ok(BuildInfo {
            commit_hash,
            branch,
            build_number,
            rust_version,
            compilation_flags: self.get_compilation_flags(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_secs(),
        })
    }

    fn collect_environment_info(&self) -> Result<EnvironmentInfo, Box<dyn std::error::Error>> {
        let ci_provider = if std::env::var("GITHUB_ACTIONS").is_ok() {
            "GitHub Actions".to_string()
        } else if std::env::var("TRAVIS").is_ok() {
            "Travis CI".to_string()
        } else if std::env::var("CIRCLECI").is_ok() {
            "CircleCI".to_string()
        } else {
            "Unknown".to_string()
        };

        let runner_os = std::env::var("RUNNER_OS").unwrap_or_else(|_| std::env::consts::OS.to_string());
        let runner_arch = std::env::var("RUNNER_ARCH").unwrap_or_else(|_| std::env::consts::ARCH.to_string());

        let environment_variables = self.collect_relevant_env_vars();
        let system_info = crate::debug::platform_detector::get_environment_info();

        Ok(EnvironmentInfo {
            ci_provider,
            runner_os,
            runner_arch,
            environment_variables,
            system_info,
        })
    }

    fn collect_artifacts(&self) -> Result<Vec<ArtifactInfo>, Box<dyn std::error::Error>> {
        let mut artifacts = Vec::new();

        // Look for crash logs
        if let Ok(entries) = fs::read_dir("crash_logs") {
            for entry in entries.flatten() {
                if let Ok(metadata) = entry.metadata() {
                    if metadata.is_file() && metadata.len() <= self.max_artifact_size {
                        artifacts.push(ArtifactInfo {
                            name: entry.file_name().to_string_lossy().to_string(),
                            path: entry.path().to_string_lossy().to_string(),
                            size_bytes: metadata.len(),
                            artifact_type: ArtifactType::CrashLog,
                            description: "Crash log file".to_string(),
                        });
                    }
                }
            }
        }

        // Look for test crash logs
        if let Ok(entries) = fs::read_dir("test_crash_logs") {
            for entry in entries.flatten() {
                if let Ok(metadata) = entry.metadata() {
                    if metadata.is_file() && metadata.len() <= self.max_artifact_size {
                        artifacts.push(ArtifactInfo {
                            name: entry.file_name().to_string_lossy().to_string(),
                            path: entry.path().to_string_lossy().to_string(),
                            size_bytes: metadata.len(),
                            artifact_type: ArtifactType::TestOutput,
                            description: "Test crash log".to_string(),
                        });
                    }
                }
            }
        }

        // Look for core dumps (platform-specific)
        #[cfg(unix)]
        {
            let core_patterns = ["/cores/core.*", "./core.*", "/tmp/core.*"];
            for pattern in &core_patterns {
                // In a real implementation, we'd use glob patterns to find core dumps
                // For now, we'll check if common core dump files exist
                let path = Path::new(pattern.trim_start_matches('*'));
                if path.exists() {
                    if let Ok(metadata) = path.metadata() {
                        if metadata.len() <= self.max_artifact_size {
                            artifacts.push(ArtifactInfo {
                                name: path.file_name().unwrap().to_string_lossy().to_string(),
                                path: path.to_string_lossy().to_string(),
                                size_bytes: metadata.len(),
                                artifact_type: ArtifactType::CoreDump,
                                description: "Core dump file".to_string(),
                            });
                        }
                    }
                }
            }
        }

        // Look for built binaries with debug symbols
        let binary_path = Path::new("target/debug/lambdust");
        if binary_path.exists() {
            if let Ok(metadata) = binary_path.metadata() {
                artifacts.push(ArtifactInfo {
                    name: "lambdust-debug".to_string(),
                    path: binary_path.to_string_lossy().to_string(),
                    size_bytes: metadata.len(),
                    artifact_type: ArtifactType::Binary,
                    description: "Debug binary with symbols".to_string(),
                });
            }
        }

        Ok(artifacts)
    }

    fn format_crash_report_markdown(&self, report: &CrashReport) -> Result<String, Box<dyn std::error::Error>> {
        let mut content = String::new();
        
        content.push_str("# Crash Report\n\n");
        
        // Build information
        content.push_str("## Build Information\n\n");
        content.push_str(&format!("- **Commit**: {}\n", report.build_info.commit_hash));
        content.push_str(&format!("- **Branch**: {}\n", report.build_info.branch));
        content.push_str(&format!("- **Build Number**: {}\n", report.build_info.build_number));
        content.push_str(&format!("- **Rust Version**: {}\n", report.build_info.rust_version));
        content.push_str(&format!("- **Timestamp**: {}\n\n", report.build_info.timestamp));
        
        // Environment information
        content.push_str("## Environment\n\n");
        content.push_str(&format!("- **CI Provider**: {}\n", report.environment.ci_provider));
        content.push_str(&format!("- **OS**: {}\n", report.environment.runner_os));
        content.push_str(&format!("- **Architecture**: {}\n\n", report.environment.runner_arch));
        
        // Crash summary
        content.push_str("## Crash Summary\n\n");
        content.push_str(&format!("- **Total Crashes**: {}\n", report.crashes.len()));
        
        let signal_counts = self.count_signals(&report.crashes);
        for (signal, count) in signal_counts {
            let signal_name = match signal {
                libc::SIGSEGV => "SIGSEGV (Segmentation Fault)",
                libc::SIGABRT => "SIGABRT (Abort)",
                libc::SIGILL => "SIGILL (Illegal Instruction)",
                libc::SIGFPE => "SIGFPE (Floating Point Exception)",
                #[cfg(not(windows))]
                libc::SIGBUS => "SIGBUS (Bus Error)",
                -1 => "PANIC",
                _ => "Unknown Signal",
            };
            content.push_str(&format!("- **{}**: {} occurrences\n", signal_name, count));
        }
        content.push('\n');
        
        // Pattern analysis
        if !report.patterns.is_empty() {
            content.push_str("## Crash Patterns\n\n");
            for pattern in &report.patterns {
                content.push_str(&format!("### {}\n\n", pattern));
                
                // Find corresponding suggestion
                if let Some(suggestion) = report.suggestions.iter().find(|s| s.pattern == *pattern) {
                    content.push_str(&format!("**Suggested Fix**: {}\n\n", suggestion.suggested_fix));
                    if let Some(example) = &suggestion.code_example {
                        content.push_str("**Example**:\n```rust\n");
                        content.push_str(example);
                        content.push_str("\n```\n\n");
                    }
                }
            }
        }
        
        // Test summary
        if let Some(ref summary) = report.test_summary {
            content.push_str("## Test Results\n\n");
            content.push_str(&format!("{}\n", summary));
        }
        
        // Artifacts
        if !report.artifacts.is_empty() {
            content.push_str("## Available Artifacts\n\n");
            for artifact in &report.artifacts {
                content.push_str(&format!("- **{}** ({} bytes): {}\n", 
                                        artifact.name, artifact.size_bytes, artifact.description));
            }
            content.push('\n');
        }
        
        Ok(content)
    }

    fn format_crash_report_json(&self, report: &CrashReport) -> Result<String, Box<dyn std::error::Error>> {
        // In a real implementation, we'd use serde_json for proper serialization
        // For now, we'll create a simple JSON structure
        let mut json = String::new();
        json.push_str("{\n");
        json.push_str(&format!("  \"crash_count\": {},\n", report.crashes.len()));
        json.push_str(&format!("  \"pattern_count\": {},\n", report.patterns.len()));
        json.push_str(&format!("  \"build_info\": {{\n"));
        json.push_str(&format!("    \"commit\": \"{}\",\n", report.build_info.commit_hash));
        json.push_str(&format!("    \"branch\": \"{}\",\n", report.build_info.branch));
        json.push_str(&format!("    \"timestamp\": {}\n", report.build_info.timestamp));
        json.push_str("  },\n");
        json.push_str(&format!("  \"environment\": {{\n"));
        json.push_str(&format!("    \"ci_provider\": \"{}\",\n", report.environment.ci_provider));
        json.push_str(&format!("    \"os\": \"{}\",\n", report.environment.runner_os));
        json.push_str(&format!("    \"arch\": \"{}\"\n", report.environment.runner_arch));
        json.push_str("  }\n");
        json.push_str("}\n");
        
        Ok(json)
    }

    fn write_patterns_analysis(
        &self,
        writer: &mut dyn Write,
        patterns: &[CrashPattern],
        suggestions: &[FixSuggestion],
    ) -> Result<(), Box<dyn std::error::Error>> {
        writeln!(writer, "# Crash Pattern Analysis\n")?;
        
        if patterns.is_empty() {
            writeln!(writer, "No crash patterns identified.\n")?;
            return Ok(());
        }
        
        writeln!(writer, "## Identified Patterns\n")?;
        
        for pattern in patterns {
            writeln!(writer, "### {}\n", pattern)?;
            
            // Find corresponding suggestions
            let relevant_suggestions: Vec<_> = suggestions.iter()
                .filter(|s| s.pattern == *pattern)
                .collect();
            
            if !relevant_suggestions.is_empty() {
                writeln!(writer, "#### Suggestions\n")?;
                for suggestion in relevant_suggestions {
                    writeln!(writer, "{}\n", suggestion)?;
                }
            }
        }
        
        Ok(())
    }

    fn write_github_annotations(&self, path: &Path, report: &CrashReport) -> Result<(), Box<dyn std::error::Error>> {
        let mut content = String::new();
        
        // GitHub Actions workflow commands for annotations
        if !report.crashes.is_empty() {
            content.push_str(&format!("::error::Found {} crashes in build\n", report.crashes.len()));
        }
        
        for pattern in &report.patterns {
            match pattern {
                CrashPattern::NullPointerDereference { location, frequency } => {
                    content.push_str(&format!("::error file={}::Null pointer dereference ({} occurrences)\n", 
                                            self.extract_filename(location), frequency));
                }
                CrashPattern::UseAfterFree { location, frequency, .. } => {
                    content.push_str(&format!("::error file={}::Use after free ({} occurrences)\n", 
                                            self.extract_filename(location), frequency));
                }
                CrashPattern::UnsafeCode { location, frequency, unsafe_operation } => {
                    content.push_str(&format!("::warning file={}::Unsafe {} ({} occurrences)\n", 
                                            self.extract_filename(location), unsafe_operation, frequency));
                }
                _ => {
                    content.push_str(&format!("::notice::Detected crash pattern: {}\n", pattern));
                }
            }
        }
        
        fs::write(path, content)?;
        Ok(())
    }

    fn copy_artifact(&self, artifact: &ArtifactInfo) -> Result<PathBuf, Box<dyn std::error::Error>> {
        let source = Path::new(&artifact.path);
        let dest = self.output_dir.join(&artifact.name);
        
        fs::copy(source, &dest)?;
        
        // Compress if enabled and file is large
        if self.enable_artifact_compression && artifact.size_bytes > 1024 * 1024 {
            // In a real implementation, we'd compress the file
            // For now, we'll just return the original file
        }
        
        Ok(dest)
    }

    fn get_git_commit_hash(&self) -> Result<String, Box<dyn std::error::Error>> {
        let output = Command::new("git").arg("rev-parse").arg("HEAD").output()?;
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    fn get_git_branch(&self) -> Result<String, Box<dyn std::error::Error>> {
        let output = Command::new("git").arg("rev-parse").arg("--abbrev-ref").arg("HEAD").output()?;
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    fn get_rust_version(&self) -> Result<String, Box<dyn std::error::Error>> {
        let output = Command::new("rustc").arg("--version").output()?;
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    fn get_compilation_flags(&self) -> Vec<String> {
        let mut flags = Vec::new();
        
        // Check for common Rust flags
        if cfg!(debug_assertions) {
            flags.push("debug_assertions".to_string());
        }
        
        if cfg!(target_feature = "avx2") {
            flags.push("avx2".to_string());
        }
        
        flags
    }

    fn collect_relevant_env_vars(&self) -> HashMap<String, String> {
        let mut vars = HashMap::new();
        
        let relevant_vars = [
            "GITHUB_ACTIONS", "GITHUB_WORKFLOW", "GITHUB_RUN_ID", "GITHUB_RUN_NUMBER",
            "RUNNER_OS", "RUNNER_ARCH", "RUNNER_NAME",
            "RUST_BACKTRACE", "RUST_LOG", "CARGO_CFG_TARGET_ARCH", "CARGO_CFG_TARGET_OS",
        ];
        
        for var in &relevant_vars {
            if let Ok(value) = std::env::var(var) {
                vars.insert(var.to_string(), value);
            }
        }
        
        vars
    }

    fn count_signals(&self, crashes: &[CrashInfo]) -> HashMap<i32, usize> {
        let mut counts = HashMap::new();
        
        for crash in crashes {
            *counts.entry(crash.signal).or_insert(0) += 1;
        }
        
        counts
    }

    fn extract_filename(&self, location: &str) -> String {
        // Extract filename from location string for GitHub annotations
        if location.contains("src/") {
            if let Some(start) = location.find("src/") {
                if let Some(end) = location[start..].find("::") {
                    return location[start..start + end].to_string();
                }
            }
        }
        
        "unknown".to_string()
    }
}

/// Upload crash artifacts to CI system
pub fn upload_crash_artifacts(artifacts: &[PathBuf]) -> Result<(), Box<dyn std::error::Error>> {
    println!("Uploading {} crash artifacts", artifacts.len());
    
    // For GitHub Actions, we can use the upload-artifact action
    // This would typically be called from a workflow step
    
    for artifact_path in artifacts {
        println!("  - {}", artifact_path.display());
    }
    
    // In a real implementation, this would integrate with the CI system's artifact upload API
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_crash_report_generation() {
        let temp_dir = tempdir().unwrap();
        let reporter = CiReporter::new(temp_dir.path().to_path_buf());
        
        // Create a simple crash report
        let crashes = vec![]; // Empty for test
        let patterns = vec![];
        let suggestions = vec![];
        
        let report = reporter.generate_crash_report(crashes, patterns, suggestions, None);
        assert!(report.is_ok());
    }

    #[test]
    fn test_markdown_formatting() {
        let temp_dir = tempdir().unwrap();
        let reporter = CiReporter::new(temp_dir.path().to_path_buf());
        
        let report = CrashReport {
            build_info: BuildInfo {
                commit_hash: "abc123".to_string(),
                branch: "main".to_string(),
                build_number: "42".to_string(),
                rust_version: "1.82.0".to_string(),
                compilation_flags: vec!["debug".to_string()],
                timestamp: 1234567890,
            },
            environment: EnvironmentInfo {
                ci_provider: "GitHub Actions".to_string(),
                runner_os: "Ubuntu".to_string(),
                runner_arch: "X64".to_string(),
                environment_variables: HashMap::new(),
                system_info: HashMap::new(),
            },
            crashes: vec![],
            patterns: vec![],
            suggestions: vec![],
            test_summary: None,
            artifacts: vec![],
        };
        
        let markdown = reporter.format_crash_report_markdown(&report);
        assert!(markdown.is_ok());
        assert!(markdown.unwrap().contains("# Crash Report"));
    }
}