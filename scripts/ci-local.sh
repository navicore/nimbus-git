#!/bin/bash
# Local CI script that matches GitHub Actions CI
set -e

echo "🔍 Running CI checks..."

echo "📝 Checking formatting..."
cargo fmt --all -- --check

echo "🔎 Running clippy..."
cargo clippy --all -- -D warnings

echo "🧪 Running tests..."
cargo test --all

echo "🔒 Running security audit..."
cargo audit || true

echo "✅ All CI checks passed!"