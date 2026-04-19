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

## Building

### Prerequisites

- Rust 1.56+ 
- Linux OS (or macOS with cross-compilation)

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

# Build
make build-arm-release
```

**Using Make:**
```bash
make build-arm        # Debug build
make build-arm-release # Release build
make deploy            # Deploy to Raspberry Pi
```

**Direct cargo:**
```bash
# Using the build script
./scripts/build-arm.sh

# Or manually
cargo build --target armv7-unknown-linux-gnueabihf --release
```

### Deployment

```bash
# Copy binary to Raspberry Pi
make deploy

# Or manually
rsync -avz target/armv7-unknown-linux-gnueabihf/release/ pi@raspberrypi.local:/home/pi/rpi-sim868/
```

## CI/CD

This project uses GitHub Actions for automated builds:
- **Linting**: `cargo fmt` and `cargo clippy` on every PR
- **Cross-compilation**: Automatic ARM builds on every push
- **Releases**: Automated releases when tags are pushed

See `.github/workflows/rust.yml` for details.

## Tested SIM868 UART selection switch: 
- **A** - `ttyUSBx` port 
- **B** - `ttySx` port.

## Tested devices: 
- RPi 3 Model B
- RPi 4 Model B 
- RPi Zero W
- RPi Zero 2 W

## License

MIT License - see [LICENSE](LICENSE) file for details.
