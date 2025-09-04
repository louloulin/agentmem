#!/bin/bash

# AgentMem Continuous Improvement Script
# Automated technical debt management, performance optimization, and quality assurance

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
REPORTS_DIR="$PROJECT_ROOT/reports"
TOOLS_DIR="$PROJECT_ROOT/tools"

echo -e "${BLUE}🚀 AgentMem Continuous Improvement Suite${NC}"
echo -e "${BLUE}=======================================${NC}"
echo "Project Root: $PROJECT_ROOT"
echo "Reports Directory: $REPORTS_DIR"
echo ""

# Create reports directory
mkdir -p "$REPORTS_DIR"

# Function to print section headers
print_section() {
    echo -e "\n${BLUE}$1${NC}"
    echo -e "${BLUE}$(printf '=%.0s' $(seq 1 ${#1}))${NC}"
}

# Function to print success messages
print_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

# Function to print warning messages
print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

# Function to print error messages
print_error() {
    echo -e "${RED}❌ $1${NC}"
}

# 1. Code Quality Analysis
print_section "📊 Code Quality Analysis"

echo "Running Rust code analysis..."
cd "$PROJECT_ROOT"

# Run clippy for linting
echo "🔍 Running clippy analysis..."
if cargo clippy --all-targets --all-features -- -D warnings > "$REPORTS_DIR/clippy_report.txt" 2>&1; then
    print_success "Clippy analysis completed successfully"
else
    print_warning "Clippy found issues - check $REPORTS_DIR/clippy_report.txt"
fi

# Run rustfmt check
echo "🎨 Checking code formatting..."
if cargo fmt -- --check > "$REPORTS_DIR/fmt_report.txt" 2>&1; then
    print_success "Code formatting is consistent"
else
    print_warning "Code formatting issues found - run 'cargo fmt' to fix"
fi

# Run security audit
echo "🔒 Running security audit..."
if command -v cargo-audit &> /dev/null; then
    if cargo audit > "$REPORTS_DIR/security_audit.txt" 2>&1; then
        print_success "Security audit passed"
    else
        print_warning "Security vulnerabilities found - check $REPORTS_DIR/security_audit.txt"
    fi
else
    print_warning "cargo-audit not installed - run 'cargo install cargo-audit'"
fi

# Run dependency check
echo "📦 Checking dependencies..."
if command -v cargo-outdated &> /dev/null; then
    cargo outdated > "$REPORTS_DIR/outdated_deps.txt" 2>&1
    print_success "Dependency check completed"
else
    print_warning "cargo-outdated not installed - run 'cargo install cargo-outdated'"
fi

# 2. Test Coverage Analysis
print_section "🧪 Test Coverage Analysis"

echo "Running test suite with coverage..."
if command -v cargo-tarpaulin &> /dev/null; then
    if cargo tarpaulin --out Html --output-dir "$REPORTS_DIR" > "$REPORTS_DIR/coverage.log" 2>&1; then
        print_success "Test coverage report generated"
    else
        print_warning "Coverage analysis failed - check $REPORTS_DIR/coverage.log"
    fi
else
    print_warning "cargo-tarpaulin not installed - run 'cargo install cargo-tarpaulin'"
fi

# Run regular tests
echo "🧪 Running test suite..."
if cargo test --workspace > "$REPORTS_DIR/test_results.txt" 2>&1; then
    print_success "All tests passed"
else
    print_error "Some tests failed - check $REPORTS_DIR/test_results.txt"
fi

# 3. Performance Benchmarking
print_section "⚡ Performance Benchmarking"

echo "Building performance benchmark tool..."
if [ -d "$TOOLS_DIR/performance-benchmark" ]; then
    cd "$TOOLS_DIR/performance-benchmark"
    if cargo build --release > "$REPORTS_DIR/benchmark_build.log" 2>&1; then
        print_success "Benchmark tool built successfully"
        
        echo "Running performance benchmarks..."
        if ./target/release/benchmark > "$REPORTS_DIR/benchmark_output.txt" 2>&1; then
            print_success "Performance benchmarks completed"
            # Move generated reports to reports directory
            [ -f "performance_report.md" ] && mv performance_report.md "$REPORTS_DIR/"
            [ -f "performance_results.json" ] && mv performance_results.json "$REPORTS_DIR/"
        else
            print_warning "Performance benchmarks failed - check $REPORTS_DIR/benchmark_output.txt"
        fi
    else
        print_warning "Failed to build benchmark tool - check $REPORTS_DIR/benchmark_build.log"
    fi
    cd "$PROJECT_ROOT"
else
    print_warning "Performance benchmark tool not found"
fi

# 4. Documentation Generation
print_section "📚 Documentation Generation"

echo "Generating API documentation..."
if cargo doc --no-deps --all-features > "$REPORTS_DIR/doc_generation.log" 2>&1; then
    print_success "API documentation generated"
else
    print_warning "Documentation generation failed - check $REPORTS_DIR/doc_generation.log"
fi

# Check for missing documentation
echo "Checking for missing documentation..."
if cargo doc --no-deps --all-features 2>&1 | grep -i "warning.*missing" > "$REPORTS_DIR/missing_docs.txt"; then
    print_warning "Missing documentation found - check $REPORTS_DIR/missing_docs.txt"
else
    print_success "All public items are documented"
fi

# 5. Build Verification
print_section "🏗️  Build Verification"

echo "Testing release build..."
if cargo build --release > "$REPORTS_DIR/release_build.log" 2>&1; then
    print_success "Release build successful"
else
    print_error "Release build failed - check $REPORTS_DIR/release_build.log"
fi

echo "Testing debug build..."
if cargo build > "$REPORTS_DIR/debug_build.log" 2>&1; then
    print_success "Debug build successful"
else
    print_error "Debug build failed - check $REPORTS_DIR/debug_build.log"
fi

# 6. Docker Build Verification
print_section "🐳 Docker Build Verification"

if command -v docker &> /dev/null; then
    echo "Testing Docker build..."
    if docker build -t agentmem-test . > "$REPORTS_DIR/docker_build.log" 2>&1; then
        print_success "Docker build successful"
        
        # Clean up test image
        docker rmi agentmem-test > /dev/null 2>&1 || true
    else
        print_warning "Docker build failed - check $REPORTS_DIR/docker_build.log"
    fi
else
    print_warning "Docker not available - skipping Docker build test"
fi

# 7. License and Legal Compliance
print_section "⚖️  License and Legal Compliance"

echo "Checking license headers..."
find "$PROJECT_ROOT" -name "*.rs" -type f | while read -r file; do
    if ! head -10 "$file" | grep -q "Copyright\|License\|SPDX"; then
        echo "Missing license header: $file" >> "$REPORTS_DIR/missing_licenses.txt"
    fi
done

if [ -f "$REPORTS_DIR/missing_licenses.txt" ]; then
    print_warning "Some files missing license headers - check $REPORTS_DIR/missing_licenses.txt"
else
    print_success "All source files have proper license headers"
fi

# 8. Dependency License Check
echo "Checking dependency licenses..."
if command -v cargo-license &> /dev/null; then
    cargo license > "$REPORTS_DIR/dependency_licenses.txt" 2>&1
    print_success "Dependency license check completed"
else
    print_warning "cargo-license not installed - run 'cargo install cargo-license'"
fi

# 9. Generate Summary Report
print_section "📋 Generating Summary Report"

SUMMARY_FILE="$REPORTS_DIR/improvement_summary.md"
cat > "$SUMMARY_FILE" << EOF
# AgentMem Continuous Improvement Report

**Generated:** $(date)
**Project Root:** $PROJECT_ROOT

## Summary

This report contains the results of automated code quality analysis, performance benchmarking, and compliance checks.

## Code Quality

- **Clippy Analysis:** $([ -f "$REPORTS_DIR/clippy_report.txt" ] && echo "✅ Completed" || echo "❌ Failed")
- **Code Formatting:** $(cargo fmt -- --check > /dev/null 2>&1 && echo "✅ Consistent" || echo "⚠️ Issues Found")
- **Security Audit:** $([ -f "$REPORTS_DIR/security_audit.txt" ] && echo "✅ Completed" || echo "⚠️ Skipped")

## Testing

- **Test Suite:** $(cargo test --workspace > /dev/null 2>&1 && echo "✅ All Passed" || echo "❌ Some Failed")
- **Coverage Report:** $([ -f "$REPORTS_DIR/tarpaulin-report.html" ] && echo "✅ Generated" || echo "⚠️ Not Available")

## Performance

- **Benchmarks:** $([ -f "$REPORTS_DIR/performance_report.md" ] && echo "✅ Completed" || echo "⚠️ Not Run")

## Documentation

- **API Docs:** $(cargo doc --no-deps > /dev/null 2>&1 && echo "✅ Generated" || echo "❌ Failed")

## Build Verification

- **Release Build:** $(cargo build --release > /dev/null 2>&1 && echo "✅ Success" || echo "❌ Failed")
- **Docker Build:** $([ -f "$REPORTS_DIR/docker_build.log" ] && echo "✅ Tested" || echo "⚠️ Skipped")

## Compliance

- **License Headers:** $([ ! -f "$REPORTS_DIR/missing_licenses.txt" ] && echo "✅ Complete" || echo "⚠️ Issues Found")
- **Dependency Licenses:** $([ -f "$REPORTS_DIR/dependency_licenses.txt" ] && echo "✅ Checked" || echo "⚠️ Not Checked")

## Next Steps

1. Review all generated reports in the \`reports/\` directory
2. Address any warnings or errors found
3. Update dependencies if outdated versions are detected
4. Improve test coverage if below target threshold
5. Add missing documentation for public APIs

## Files Generated

EOF

# List all generated report files
find "$REPORTS_DIR" -name "*.txt" -o -name "*.html" -o -name "*.md" -o -name "*.json" | sort | while read -r file; do
    echo "- \`$(basename "$file")\`" >> "$SUMMARY_FILE"
done

print_success "Summary report generated: $SUMMARY_FILE"

# 10. Final Summary
print_section "🎯 Improvement Summary"

echo "Reports generated in: $REPORTS_DIR"
echo ""
echo "Key files to review:"
echo "  📊 improvement_summary.md - Overall summary"
echo "  🔍 clippy_report.txt - Code quality issues"
echo "  🧪 test_results.txt - Test execution results"
echo "  ⚡ performance_report.md - Performance benchmarks"
echo "  📚 API documentation in target/doc/"
echo ""

# Count issues
TOTAL_ISSUES=0
[ -f "$REPORTS_DIR/clippy_report.txt" ] && CLIPPY_ISSUES=$(grep -c "warning\|error" "$REPORTS_DIR/clippy_report.txt" 2>/dev/null || echo 0) && TOTAL_ISSUES=$((TOTAL_ISSUES + CLIPPY_ISSUES))
[ -f "$REPORTS_DIR/missing_licenses.txt" ] && LICENSE_ISSUES=$(wc -l < "$REPORTS_DIR/missing_licenses.txt" 2>/dev/null || echo 0) && TOTAL_ISSUES=$((TOTAL_ISSUES + LICENSE_ISSUES))

if [ $TOTAL_ISSUES -eq 0 ]; then
    print_success "No critical issues found! 🎉"
else
    print_warning "$TOTAL_ISSUES issues found - please review the reports"
fi

echo ""
print_success "Continuous improvement analysis completed!"
echo -e "${BLUE}Next run this script regularly to maintain code quality.${NC}"
