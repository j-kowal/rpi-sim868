# RPi SIM868

<img align="right" style="width:40%;" src="https://github.com/j-kowal/rpi-sim868/assets/23199671/6671900e-0038-42e0-84d9-8a0091c00d39" alt="hat"/>

[![crates.io](https://img.shields.io/crates/v/rpi_sim868)](https://crates.io/crates/rpi_sim868)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![Minimum rustc version](https://img.shields.io/badge/rustc-v1.56.0-blue.svg)](https://blog.rust-lang.org/2021/10/21/Rust-1.56.0.html)
[![Rust CI](https://github.com/j-kowal/rpi-sim868/actions/workflows/rust.yml/badge.svg)](https://github.com/j-kowal/rpi-sim868/actions/workflows/rust.yml)

### [Documentation](https://docs.rs/rpi_sim868)

RPi SIM868 is a Rust crate designed to simplify interaction with the [Waveshare SIM868 HAT](https://www.waveshare.com/gsm-gprs-gnss-hat.htm) for Raspberry Pi. It utilizes the [Tokio](https://tokio.rs) runtime for managing asynchronous tasks and includes its own task scheduler based on a priority queue.

Each method call initiates a new task, which is enqueued with a priority to ensure swift execution as soon as the serial port becomes available. 

Methods (except for `HAT::turn_on`) return `TaskJoinHandle<T>`, where `T` represents the type resulting from parsing and analyzing the serial output, if applicable. Tasks related to phone calls are treated as first-class citizens with high priority, reducing delays in answering or concluding calls.

RPi SIM868 was conceived following a high-altitude balloon launch where the HAT served as a backup tracking device. The initial software, written in Python, lacked the performance and safety synonymous with Rust.

## Features

- **Async Task Scheduler**: Priority queue-based task management with HIGH and NORMAL priorities
- **Phone Call Priority**: Incoming/outgoing calls are handled with highest priority
- **Cross-Platform**: Build natively on Raspberry Pi or cross-compile from macOS/Linux
- **Modular Design**: Separate modules for HAT, SMS, GNSS, GPRS, and Phone functionality
- **Comprehensive Testing**: 52+ unit tests covering priority queue, parsing, and error handling

## Building

### Prerequisites

- Rust 1.56+ 
- Linux OS for native builds (or macOS with cross-compilation)

### Quick Start

#### Option 1: Build on Raspberry Pi (Native)

```bash
# On your Raspberry Pi
cargo build --release
```

#### Option 2: Cross-compile from macOS/Linux

**macOS Setup:**
```bash
# Install ARM cross-compilation toolchain
brew tap messense/macos-cross-toolchains
brew install armv7-unknown-linux-gnueabihf

# Add Rust target
rustup target add armv7-unknown-linux-gnueabihf

# Build using Make
make build-arm-release

# Or use the build script
./scripts/build-arm.sh release
```

**Linux Setup:**
```bash
# Install ARM cross-compiler
sudo apt-get install gcc-arm-linux-gnueabihf

# Add Rust target
rustup target add armv7-unknown-linux-gnueabihf

# Build
cargo build --target armv7-unknown-linux-gnueabihf --release
```

**Using Make:**
```bash
make build-arm        # Debug build
make build-arm-release # Release build
make deploy            # Deploy to Raspberry Pi
```

### Deployment

```bash
# Copy binary to Raspberry Pi
make deploy

# Or manually with rsync
rsync -avz target/armv7-unknown-linux-gnueabihf/release/ pi@raspberrypi.local:/home/pi/rpi-sim868/

# Or with scp
scp target/armv7-unknown-linux-gnueabihf/release/*.so pi@raspberrypi.local:/home/pi/
```

## Testing

The project includes 52+ comprehensive unit tests covering:

- **Priority Queue** (12 tests): FIFO ordering, priority handling, task removal
- **AT Command Parsing** (16 tests): Response parsing, error detection, edge cases
- **Error Handling** (13 tests): Error propagation, conversions, recovery strategies
- **Async Integration** (11 tests): Concurrent operations, timeouts, cleanup

### Running Tests

```bash
# Run all tests (requires Linux or WSL - uses mock feature on macOS)
cargo test --tests --no-default-features --features mock

# Run specific test module
cargo test --test priority_queue_tests --no-default-features --features mock
cargo test --test parser_tests --no-default-features --features mock
cargo test --test error_tests --no-default-features --features mock
cargo test --test integration_tests --no-default-features --features mock

# Run with output
cargo test --tests --no-default-features --features mock -- --nocapture
```

### Feature Flags

- `hardware` (default): Enables hardware support via rppal (Raspberry Pi only)
- `mock`: Enables mock implementations for testing without hardware

```bash
# Build without hardware support (for testing)
cargo build --no-default-features --features mock

# Build with hardware support (for Raspberry Pi)
cargo build --release
```

## CI/CD

This project uses GitHub Actions for automated builds:
- **Linting**: `cargo fmt` and `cargo clippy` on every PR
- **Cross-compilation**: Automatic ARM builds on every push
- **Testing**: All unit tests run on every PR
- **Releases**: Automated releases when tags are pushed

See `.github/workflows/rust.yml` for details.

### Creating a Release

To create a new release, push a tag to GitHub:

```bash
# Create and push a new tag (follows semantic versioning)
git tag -a v0.2.0 -m "Release version 0.2.0"
git push origin v0.2.0
```

GitHub Actions will automatically:
1. Build the release binary for ARM
2. Run all tests
3. Create a GitHub Release with:
   - Binary tarball (.tar.gz)
   - Auto-generated release notes
   - README and LICENSE

Releases are available at: https://github.com/j-kowal/rpi-sim868/releases

## Tested SIM868 UART Selection Switch

- **A** - `ttyUSBx` port (USB interface)
- **B** - `ttySx` port (UART interface)

## Tested Devices

- Raspberry Pi 3 Model B
- Raspberry Pi 4 Model B 
- Raspberry Pi Zero W
- Raspberry Pi Zero 2 W

## Architecture

### Task Priority System

Tasks are scheduled using a priority queue with two levels:
- **HIGH**: Phone calls (incoming/outgoing), urgent operations
- **NORMAL**: SMS, GNSS, GPRS, general HAT operations

### Modules

- `hat`: HAT control (power, signal strength, network status)
- `sms`: SMS sending and receiving
- `gnss`: GPS/GNSS positioning and tracking
- `gprs`: HTTP requests and data connectivity
- `phone`: Voice calls (dial, answer, hangup)

## License

MIT License - see [LICENSE](LICENSE) file for details.

## Contributing

Contributions are welcome! Please ensure:
1. Code follows `cargo fmt` formatting
2. All tests pass: `cargo test`
3. Clippy warnings are addressed: `cargo clippy`
4. Changes are documented in PR description
