.PHONY: all build build-release check fmt lint clean test deploy help

# Default target
all: check build

# Build for local development (won't work on macOS due to rppal)
build:
	@echo "Building for local target (Linux only)..."
	cargo build --all-features

# Build for Raspberry Pi (ARM)
build-arm:
	@echo "Building for Raspberry Pi (ARM)..."
	./scripts/build-arm.sh

# Build release for Raspberry Pi
build-arm-release:
	@echo "Building release for Raspberry Pi (ARM)..."
	./scripts/build-arm.sh release

# Check code without building
check:
	cargo check --all-features

# Format code
fmt:
	cargo fmt

# Run linter
lint:
	cargo clippy --all-features -- -D warnings

# Clean build artifacts
clean:
	cargo clean

# Run tests (if any)
test:
	cargo test --all-features

# Deploy to Raspberry Pi (requires PI_HOST env var or default to raspberrypi.local)
PI_HOST ?= raspberrypi.local
PI_USER ?= pi
PI_PATH ?= /home/pi/rpi-sim868

deploy: build-arm-release
	@echo "Deploying to $(PI_USER)@$(PI_HOST):$(PI_PATH)"
	ssh $(PI_USER)@$(PI_HOST) "mkdir -p $(PI_PATH)"
	rsync -avz --delete target/armv7-unknown-linux-gnueabihf/release/ $(PI_USER)@$(PI_HOST):$(PI_PATH)/
	@echo "Deployed successfully!"

# Show help
help:
	@echo "Available targets:"
	@echo "  make build           - Build for local target (Linux only)"
	@echo "  make build-arm       - Build for Raspberry Pi (ARM)"
	@echo "  make build-arm-release - Build release for Raspberry Pi"
	@echo "  make check           - Check code without building"
	@echo "  make fmt             - Format code"
	@echo "  make lint            - Run linter"
	@echo "  make clean           - Clean build artifacts"
	@echo "  make test            - Run tests"
	@echo "  make deploy          - Deploy to Raspberry Pi (set PI_HOST, PI_USER, PI_PATH)"
	@echo "  make help            - Show this help"
