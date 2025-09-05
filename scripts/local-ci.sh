#!/bin/bash
# Local CI script that mirrors GitHub Actions CI steps
# This ensures local development matches CI environment

set -e

echo "🔍 Running local CI checks..."

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ] || [ ! -d "crates" ]; then
    echo -e "${RED}Error: Must run from nimbus-git root directory${NC}"
    exit 1
fi

echo -e "${YELLOW}📝 Checking formatting...${NC}"
cargo fmt -- --check
if [ $? -eq 0 ]; then
    echo -e "${GREEN}✓ Formatting check passed${NC}"
else
    echo -e "${RED}✗ Formatting issues found. Run 'cargo fmt' to fix.${NC}"
    exit 1
fi

echo -e "${YELLOW}📦 Building all packages...${NC}"
cargo build --all
if [ $? -eq 0 ]; then
    echo -e "${GREEN}✓ Build succeeded${NC}"
else
    echo -e "${RED}✗ Build failed${NC}"
    exit 1
fi

echo -e "${YELLOW}🔍 Running clippy...${NC}"
cargo clippy --all-targets --all-features -- -W clippy::all
if [ $? -eq 0 ]; then
    echo -e "${GREEN}✓ Clippy check passed${NC}"
else
    echo -e "${RED}✗ Clippy warnings found${NC}"
    exit 1
fi

echo -e "${YELLOW}🧪 Running tests...${NC}"
cargo test --all
if [ $? -eq 0 ]; then
    echo -e "${GREEN}✓ All tests passed${NC}"
else
    echo -e "${RED}✗ Tests failed${NC}"
    exit 1
fi

echo -e "${YELLOW}🏗️ Building release binary...${NC}"
cargo build --release
if [ $? -eq 0 ]; then
    echo -e "${GREEN}✓ Release build succeeded${NC}"
else
    echo -e "${RED}✗ Release build failed${NC}"
    exit 1
fi

echo -e "${GREEN}✅ All local CI checks passed!${NC}"
echo -e "${GREEN}You can now commit and push with confidence.${NC}"