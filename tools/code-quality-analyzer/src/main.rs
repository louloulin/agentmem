//! AgentMem Code Quality Analyzer
//! 
//! Comprehensive code quality analysis tool for technical debt management,
//! performance monitoring, and code optimization recommendations.

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use regex::Regex;
use chrono::{DateTime, Utc};

/// Code quality metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityMetrics {
    pub file_path: String,
    pub lines_of_code: usize,
    pub cyclomatic_complexity: usize,
    pub technical_debt_score: f64,
    pub maintainability_index: f64,
    pub test_coverage: f64,
    pub warnings: Vec<QualityWarning>,
    pub suggestions: Vec<OptimizationSuggestion>,
}

/// Quality warning types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityWarning {
    pub warning_type: WarningType,
    pub line_number: usize,
    pub message: String,
    pub severity: Severity,
}

/// Warning types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WarningType {
    UnusedImport,
    UnusedVariable,
    DeadCode,
    LongFunction,
    ComplexFunction,
    DuplicateCode,
    MissingDocumentation,
    UnsafeCode,
    PerformanceIssue,
    SecurityIssue,
}

/// Severity levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    Info,
    Warning,
    Error,
    Critical,
}

/// Optimization suggestions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationSuggestion {
    pub suggestion_type: SuggestionType,
    pub line_number: Option<usize>,
    pub description: String,
    pub impact: Impact,
    pub effort: Effort,
}

/// Suggestion types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SuggestionType {
    Refactor,
    Performance,
    Security,
    Documentation,
    Testing,
    Architecture,
}

/// Impact levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Impact {
    Low,
    Medium,
    High,
    Critical,
}

/// Effort levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Effort {
    Low,
    Medium,
    High,
}

/// Code quality analyzer
pub struct CodeQualityAnalyzer {
    project_root: PathBuf,
    ignore_patterns: HashSet<String>,
    quality_rules: Vec<QualityRule>,
}

/// Quality rule definition
#[derive(Debug, Clone)]
pub struct QualityRule {
    pub name: String,
    pub pattern: Regex,
    pub warning_type: WarningType,
    pub severity: Severity,
    pub message: String,
}

impl CodeQualityAnalyzer {
    /// Create a new code quality analyzer
    pub fn new(project_root: PathBuf) -> Self {
        let mut analyzer = Self {
            project_root,
            ignore_patterns: HashSet::new(),
            quality_rules: Vec::new(),
        };

        analyzer.initialize_default_rules();
        analyzer.initialize_ignore_patterns();
        analyzer
    }

    /// Initialize default quality rules
    fn initialize_default_rules(&mut self) {
        // Unused imports
        self.quality_rules.push(QualityRule {
            name: "unused_import".to_string(),
            pattern: Regex::new(r"warning: unused import").unwrap(),
            warning_type: WarningType::UnusedImport,
            severity: Severity::Warning,
            message: "Unused import detected".to_string(),
        });

        // Unused variables
        self.quality_rules.push(QualityRule {
            name: "unused_variable".to_string(),
            pattern: Regex::new(r"warning: unused variable").unwrap(),
            warning_type: WarningType::UnusedVariable,
            severity: Severity::Warning,
            message: "Unused variable detected".to_string(),
        });

        // Dead code
        self.quality_rules.push(QualityRule {
            name: "dead_code".to_string(),
            pattern: Regex::new(r"warning:.*is never read").unwrap(),
            warning_type: WarningType::DeadCode,
            severity: Severity::Warning,
            message: "Dead code detected".to_string(),
        });

        // Long functions (>50 lines)
        self.quality_rules.push(QualityRule {
            name: "long_function".to_string(),
            pattern: Regex::new(r"fn\s+\w+.*\{").unwrap(),
            warning_type: WarningType::LongFunction,
            severity: Severity::Info,
            message: "Function may be too long".to_string(),
        });

        // Missing documentation
        self.quality_rules.push(QualityRule {
            name: "missing_docs".to_string(),
            pattern: Regex::new(r"pub\s+(fn|struct|enum|trait)").unwrap(),
            warning_type: WarningType::MissingDocumentation,
            severity: Severity::Info,
            message: "Public item missing documentation".to_string(),
        });

        // Unsafe code
        self.quality_rules.push(QualityRule {
            name: "unsafe_code".to_string(),
            pattern: Regex::new(r"unsafe\s*\{").unwrap(),
            warning_type: WarningType::UnsafeCode,
            severity: Severity::Error,
            message: "Unsafe code block detected".to_string(),
        });
    }

    /// Initialize ignore patterns
    fn initialize_ignore_patterns(&mut self) {
        self.ignore_patterns.insert("target/".to_string());
        self.ignore_patterns.insert(".git/".to_string());
        self.ignore_patterns.insert("node_modules/".to_string());
        self.ignore_patterns.insert("*.lock".to_string());
        self.ignore_patterns.insert("*.log".to_string());
    }

    /// Analyze project code quality
    pub fn analyze_project(&self) -> Result<ProjectQualityReport, Box<dyn std::error::Error>> {
        let mut report = ProjectQualityReport {
            project_path: self.project_root.clone(),
            analyzed_at: Utc::now(),
            total_files: 0,
            total_lines: 0,
            file_metrics: Vec::new(),
            overall_score: 0.0,
            summary: QualitySummary::default(),
        };

        // Find all Rust files
        let rust_files = self.find_rust_files(&self.project_root)?;
        report.total_files = rust_files.len();

        // Analyze each file
        for file_path in rust_files {
            if let Ok(metrics) = self.analyze_file(&file_path) {
                report.total_lines += metrics.lines_of_code;
                report.file_metrics.push(metrics);
            }
        }

        // Calculate overall metrics
        report.calculate_overall_metrics();

        Ok(report)
    }

    /// Find all Rust files in the project
    fn find_rust_files(&self, dir: &Path) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
        let mut rust_files = Vec::new();
        
        if dir.is_dir() {
            for entry in fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();
                
                // Skip ignored patterns
                if self.should_ignore(&path) {
                    continue;
                }
                
                if path.is_dir() {
                    rust_files.extend(self.find_rust_files(&path)?);
                } else if path.extension().and_then(|s| s.to_str()) == Some("rs") {
                    rust_files.push(path);
                }
            }
        }
        
        Ok(rust_files)
    }

    /// Check if path should be ignored
    fn should_ignore(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy();
        self.ignore_patterns.iter().any(|pattern| {
            if pattern.ends_with('/') {
                path_str.contains(pattern)
            } else if pattern.contains('*') {
                // Simple glob matching
                let pattern = pattern.replace('*', ".*");
                Regex::new(&pattern).map_or(false, |re| re.is_match(&path_str))
            } else {
                path_str.contains(pattern)
            }
        })
    }

    /// Analyze a single file
    fn analyze_file(&self, file_path: &Path) -> Result<QualityMetrics, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(file_path)?;
        let lines: Vec<&str> = content.lines().collect();
        
        let mut metrics = QualityMetrics {
            file_path: file_path.to_string_lossy().to_string(),
            lines_of_code: lines.len(),
            cyclomatic_complexity: 0,
            technical_debt_score: 0.0,
            maintainability_index: 0.0,
            test_coverage: 0.0,
            warnings: Vec::new(),
            suggestions: Vec::new(),
        };

        // Analyze each line
        for (line_num, line) in lines.iter().enumerate() {
            self.analyze_line(line, line_num + 1, &mut metrics);
        }

        // Calculate complexity metrics
        metrics.cyclomatic_complexity = self.calculate_cyclomatic_complexity(&content);
        metrics.technical_debt_score = self.calculate_technical_debt_score(&metrics);
        metrics.maintainability_index = self.calculate_maintainability_index(&metrics);
        
        // Generate optimization suggestions
        self.generate_suggestions(&mut metrics);

        Ok(metrics)
    }

    /// Analyze a single line of code
    fn analyze_line(&self, line: &str, line_number: usize, metrics: &mut QualityMetrics) {
        for rule in &self.quality_rules {
            if rule.pattern.is_match(line) {
                let warning = QualityWarning {
                    warning_type: rule.warning_type.clone(),
                    line_number,
                    message: rule.message.clone(),
                    severity: rule.severity.clone(),
                };
                metrics.warnings.push(warning);
            }
        }
    }

    /// Calculate cyclomatic complexity
    fn calculate_cyclomatic_complexity(&self, content: &str) -> usize {
        let complexity_patterns = [
            r"if\s+", r"else\s+if\s+", r"while\s+", r"for\s+", 
            r"match\s+", r"loop\s*\{", r"\|\|", r"&&", r"\?",
        ];
        
        let mut complexity = 1; // Base complexity
        
        for pattern in &complexity_patterns {
            if let Ok(re) = Regex::new(pattern) {
                complexity += re.find_iter(content).count();
            }
        }
        
        complexity
    }

    /// Calculate technical debt score
    fn calculate_technical_debt_score(&self, metrics: &QualityMetrics) -> f64 {
        let mut debt_score = 0.0;
        
        for warning in &metrics.warnings {
            debt_score += match warning.severity {
                Severity::Info => 1.0,
                Severity::Warning => 2.0,
                Severity::Error => 5.0,
                Severity::Critical => 10.0,
            };
        }
        
        // Normalize by lines of code
        if metrics.lines_of_code > 0 {
            debt_score / metrics.lines_of_code as f64 * 100.0
        } else {
            0.0
        }
    }

    /// Calculate maintainability index
    fn calculate_maintainability_index(&self, metrics: &QualityMetrics) -> f64 {
        // Simplified maintainability index calculation
        let complexity_penalty = metrics.cyclomatic_complexity as f64 * 0.5;
        let debt_penalty = metrics.technical_debt_score * 0.3;
        let size_penalty = (metrics.lines_of_code as f64).ln() * 0.1;
        
        let base_score = 100.0;
        (base_score - complexity_penalty - debt_penalty - size_penalty).max(0.0)
    }

    /// Generate optimization suggestions
    fn generate_suggestions(&self, metrics: &mut QualityMetrics) {
        // High complexity suggestion
        if metrics.cyclomatic_complexity > 10 {
            metrics.suggestions.push(OptimizationSuggestion {
                suggestion_type: SuggestionType::Refactor,
                line_number: None,
                description: "Consider breaking down complex functions into smaller, more manageable pieces".to_string(),
                impact: Impact::High,
                effort: Effort::Medium,
            });
        }

        // High technical debt suggestion
        if metrics.technical_debt_score > 5.0 {
            metrics.suggestions.push(OptimizationSuggestion {
                suggestion_type: SuggestionType::Refactor,
                line_number: None,
                description: "Address warnings and code quality issues to reduce technical debt".to_string(),
                impact: Impact::Medium,
                effort: Effort::Low,
            });
        }

        // Large file suggestion
        if metrics.lines_of_code > 500 {
            metrics.suggestions.push(OptimizationSuggestion {
                suggestion_type: SuggestionType::Architecture,
                line_number: None,
                description: "Consider splitting large files into smaller, focused modules".to_string(),
                impact: Impact::Medium,
                effort: Effort::High,
            });
        }

        // Missing tests suggestion
        if !metrics.file_path.contains("test") && metrics.test_coverage < 50.0 {
            metrics.suggestions.push(OptimizationSuggestion {
                suggestion_type: SuggestionType::Testing,
                line_number: None,
                description: "Add unit tests to improve code coverage and reliability".to_string(),
                impact: Impact::High,
                effort: Effort::Medium,
            });
        }
    }
}

/// Project quality report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectQualityReport {
    pub project_path: PathBuf,
    pub analyzed_at: DateTime<Utc>,
    pub total_files: usize,
    pub total_lines: usize,
    pub file_metrics: Vec<QualityMetrics>,
    pub overall_score: f64,
    pub summary: QualitySummary,
}

/// Quality summary
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct QualitySummary {
    pub total_warnings: usize,
    pub critical_issues: usize,
    pub error_issues: usize,
    pub warning_issues: usize,
    pub info_issues: usize,
    pub average_complexity: f64,
    pub average_maintainability: f64,
    pub total_suggestions: usize,
}

impl ProjectQualityReport {
    /// Calculate overall project metrics
    pub fn calculate_overall_metrics(&mut self) {
        if self.file_metrics.is_empty() {
            return;
        }

        let mut total_complexity = 0;
        let mut total_maintainability = 0.0;
        let mut total_warnings = 0;
        let mut critical_count = 0;
        let mut error_count = 0;
        let mut warning_count = 0;
        let mut info_count = 0;
        let mut total_suggestions = 0;

        for metrics in &self.file_metrics {
            total_complexity += metrics.cyclomatic_complexity;
            total_maintainability += metrics.maintainability_index;
            total_warnings += metrics.warnings.len();
            total_suggestions += metrics.suggestions.len();

            for warning in &metrics.warnings {
                match warning.severity {
                    Severity::Critical => critical_count += 1,
                    Severity::Error => error_count += 1,
                    Severity::Warning => warning_count += 1,
                    Severity::Info => info_count += 1,
                }
            }
        }

        self.summary = QualitySummary {
            total_warnings,
            critical_issues: critical_count,
            error_issues: error_count,
            warning_issues: warning_count,
            info_issues: info_count,
            average_complexity: total_complexity as f64 / self.file_metrics.len() as f64,
            average_maintainability: total_maintainability / self.file_metrics.len() as f64,
            total_suggestions,
        };

        // Calculate overall score (0-100)
        self.overall_score = self.summary.average_maintainability;
    }

    /// Generate HTML report
    pub fn generate_html_report(&self) -> String {
        format!(
            r#"
<!DOCTYPE html>
<html>
<head>
    <title>AgentMem Code Quality Report</title>
    <style>
        body {{ font-family: Arial, sans-serif; margin: 20px; }}
        .header {{ background: #f0f0f0; padding: 20px; border-radius: 5px; }}
        .summary {{ display: flex; gap: 20px; margin: 20px 0; }}
        .metric {{ background: #e8f4f8; padding: 15px; border-radius: 5px; flex: 1; }}
        .file-list {{ margin-top: 20px; }}
        .file-item {{ border: 1px solid #ddd; margin: 10px 0; padding: 15px; border-radius: 5px; }}
        .warning {{ color: #ff6b35; }}
        .error {{ color: #d32f2f; }}
        .critical {{ color: #b71c1c; font-weight: bold; }}
        .suggestion {{ background: #fff3cd; padding: 10px; margin: 5px 0; border-radius: 3px; }}
    </style>
</head>
<body>
    <div class="header">
        <h1>AgentMem Code Quality Report</h1>
        <p>Generated: {}</p>
        <p>Project: {}</p>
    </div>
    
    <div class="summary">
        <div class="metric">
            <h3>Overall Score</h3>
            <h2>{:.1}</h2>
        </div>
        <div class="metric">
            <h3>Total Files</h3>
            <h2>{}</h2>
        </div>
        <div class="metric">
            <h3>Total Lines</h3>
            <h2>{}</h2>
        </div>
        <div class="metric">
            <h3>Total Issues</h3>
            <h2>{}</h2>
        </div>
    </div>
    
    <h2>Quality Summary</h2>
    <ul>
        <li class="critical">Critical Issues: {}</li>
        <li class="error">Error Issues: {}</li>
        <li class="warning">Warning Issues: {}</li>
        <li>Info Issues: {}</li>
        <li>Average Complexity: {:.1}</li>
        <li>Average Maintainability: {:.1}</li>
        <li>Total Suggestions: {}</li>
    </ul>
    
    <h2>File Details</h2>
    <div class="file-list">
        {}
    </div>
</body>
</html>
            "#,
            self.analyzed_at.format("%Y-%m-%d %H:%M:%S UTC"),
            self.project_path.display(),
            self.overall_score,
            self.total_files,
            self.total_lines,
            self.summary.total_warnings,
            self.summary.critical_issues,
            self.summary.error_issues,
            self.summary.warning_issues,
            self.summary.info_issues,
            self.summary.average_complexity,
            self.summary.average_maintainability,
            self.summary.total_suggestions,
            self.generate_file_details_html()
        )
    }

    /// Generate HTML for file details
    fn generate_file_details_html(&self) -> String {
        self.file_metrics
            .iter()
            .map(|metrics| {
                let warnings_html = metrics
                    .warnings
                    .iter()
                    .map(|w| {
                        format!(
                            r#"<div class="{}">Line {}: {}</div>"#,
                            match w.severity {
                                Severity::Critical => "critical",
                                Severity::Error => "error",
                                Severity::Warning => "warning",
                                Severity::Info => "info",
                            },
                            w.line_number,
                            w.message
                        )
                    })
                    .collect::<Vec<_>>()
                    .join("");

                let suggestions_html = metrics
                    .suggestions
                    .iter()
                    .map(|s| format!(r#"<div class="suggestion">{}</div>"#, s.description))
                    .collect::<Vec<_>>()
                    .join("");

                format!(
                    r#"
                    <div class="file-item">
                        <h3>{}</h3>
                        <p>Lines: {} | Complexity: {} | Maintainability: {:.1} | Debt Score: {:.1}</p>
                        <h4>Issues ({})</h4>
                        {}
                        <h4>Suggestions ({})</h4>
                        {}
                    </div>
                    "#,
                    metrics.file_path,
                    metrics.lines_of_code,
                    metrics.cyclomatic_complexity,
                    metrics.maintainability_index,
                    metrics.technical_debt_score,
                    metrics.warnings.len(),
                    warnings_html,
                    metrics.suggestions.len(),
                    suggestions_html
                )
            })
            .collect::<Vec<_>>()
            .join("")
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 AgentMem Code Quality Analyzer");
    println!("==================================");

    let project_root = std::env::current_dir()?.parent().unwrap().to_path_buf();
    let analyzer = CodeQualityAnalyzer::new(project_root);
    
    println!("📊 Analyzing project code quality...");
    let report = analyzer.analyze_project()?;
    
    println!("\n📈 Analysis Results:");
    println!("  Total Files: {}", report.total_files);
    println!("  Total Lines: {}", report.total_lines);
    println!("  Overall Score: {:.1}/100", report.overall_score);
    println!("  Total Issues: {}", report.summary.total_warnings);
    println!("  Critical: {}", report.summary.critical_issues);
    println!("  Errors: {}", report.summary.error_issues);
    println!("  Warnings: {}", report.summary.warning_issues);
    println!("  Average Complexity: {:.1}", report.summary.average_complexity);
    println!("  Average Maintainability: {:.1}", report.summary.average_maintainability);
    println!("  Total Suggestions: {}", report.summary.total_suggestions);

    // Generate HTML report
    let html_report = report.generate_html_report();
    fs::write("quality_report.html", html_report)?;
    println!("\n📄 HTML report generated: quality_report.html");

    // Generate JSON report
    let json_report = serde_json::to_string_pretty(&report)?;
    fs::write("quality_report.json", json_report)?;
    println!("📄 JSON report generated: quality_report.json");

    Ok(())
}
