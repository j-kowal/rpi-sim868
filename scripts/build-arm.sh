#!/bin/bash
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${YELLOW}RPi SIM868 - Cross Compilation Script${NC}"
echo "======================================"

# Target for Raspberry Pi 3/4
TARGET="armv7-unknown-linux-gnueabihf"

# Check if rustup is installed
if ! command -v rustup &> /dev/null; then
    echo -e "${RED}Error: rustup is not installed${NC}"
    echo "Install from: https://rustup.rs/"
    exit 1
fi

# Install target if not present
echo -e "${YELLOW}Checking Rust target: $TARGET${NC}"
if ! rustup target list | grep -q "$TARGET (installed)"; then
    echo "Installing target $TARGET..."
    rustup target add $TARGET
fi

# Install cross-compilation tools on macOS
if [[ "$OSTYPE" == "darwin"* ]]; then
    echo -e "${YELLOW}macOS detected - checking for ARM toolchain${NC}"
    
    # Check for Homebrew
    if ! command -v brew &> /dev/null; then
        echo -e "${RED}Homebrew is required. Install from: https://brew.sh/${NC}"
        exit 1
    fi
    
    # Install ARM toolchain if not present
    if ! command -v arm-linux-gnueabihf-gcc &> /dev/null; then
        echo "Installing ARM cross-compilation toolchain..."
        brew tap messense/macos-cross-toolchains
        brew install armv7-unknown-linux-gnueabihf
    fi
    
    # Create cargo config if it doesn't exist
    CARGO_CONFIG="$HOME/.cargo/config.toml"
    if [ ! -f "$CARGO_CONFIG" ]; then
        echo "Creating Cargo config..."
        mkdir -p "$HOME/.cargo"
        cat > "$CARGO_CONFIG" << EOF
[target.$TARGET]
linker = "arm-linux-gnueabihf-gcc"
EOF
    fi
fi

# Build mode
MODE="${1:-debug}"

echo -e "${YELLOW}Building in $MODE mode...${NC}"

if [ "$MODE" = "release" ]; then
    cargo build --target $TARGET --release --all-features
    BINARY_PATH="target/$TARGET/release"
else
    cargo build --target $TARGET --all-features
    BINARY_PATH="target/$TARGET/debug"
fi

echo -e "${GREEN}Build successful!${NC}"
echo "Binary location: $BINARY_PATH"

# List built artifacts
if [ -d "$BINARY_PATH" ]; then
    echo -e "${YELLOW}Built artifacts:${NC}"
    ls -lh "$BINARY_PATH"/*.{so,rlib} 2>/dev/null || ls -lh "$BINARY_PATH"/
fi

echo ""
echo -e "${GREEN}Next steps:${NC}"
echo "1. Copy binary to Raspberry Pi:"
echo "   scp $BINARY_PATH/*.so pi@raspberrypi.local:/home/pi/"
echo ""
echo "2. Or use rsync:"
echo "   rsync -avz $BINARY_PATH/*.so pi@raspberrypi.local:/home/pi/"
