# RPi SIM868

<img align="right" style="width:40%;" src="https://github.com/j-kowal/rpi-sim868/assets/23199671/6671900e-0038-42e0-84d9-8a0091c00d39" alt="hat"/>

[![crates.io](https://img.shields.io/crates/v/rpi_sim868)](https://crates.io/crates/rpi_sim868)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![Minimum rustc version](https://img.shields.io/badge/rustc-v1.56.0-blue.svg)](https://blog.rust-lang.org/2021/10/21/Rust-1.56.0.html)

### [Documentation](https://docs.rs/rpi_sim868)

RPi SIM868 is a crate designed to facilitate interaction with the Waveshare SIM868 HAT for Raspberry Pi. It features a non-blocking design and is well-suited for use within a multi-threaded architecture. 

The crate leverages native Rust threads and an integrated task scheduler based on a priority queue. With each interaction, a new thread is initiated and enqueued with a priority, ensuring execution as soon as the serial port becomes available. 

Each method (excluding `HAT::turn_on`) returns a `TaskJoinHandle<T>`, where `T` represents the type returned after parsing and analyzing the serial output, if applicable. Tasks related to phone calls are treated as first-class citizens with high priority, mitigating delays in answering or concluding calls.

RPi SIM868 was conceived following a high-altitude balloon launch where the HAT served as a backup tracking device. The initial software, written in Python, lacked the performance and safety synonymous with Rust.

### Tested SIM868 UART selection switch: 
- **A** - `ttyUSBx` port 
- **B** - `ttySx` port.

### Tested devices: 
- RPi 3 Model B
- RPi 4 Model B 
- RPi Zero W
- RPi Zero 2 W.
