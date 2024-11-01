# MIDKb

MIDKb is a udev daemon for Linux that binds MIDI devices to keyboard inputs. It's written in Rust and comes with a (kind of) simple configuration file.

For example, you can use your drumpad as input for SDVX's keys, and bind your CC knobs to the FX knobs.

I wrote this just for that above use case actually. I wanted to play USC with my MIDI controller, but turns out that game does not support MIDI input, only keyboard. So I wrote this program to convert MIDI signals to keyboard presses.

## Features
- Bind MIDI notes to keyboard keys
- Bind MIDI CC to keyboard keys

## Installation

1. [Install Rust](https://rustup.rs/)
2. Install dependencies:
   ```sh
   sudo dnf install alsa-lib-devel udev-devel
   ```
3. Clone the repository
4. Run the project:
   ```sh
   cargo run --release
   ```

## Configuration
Please refer to the `config.toml` file included in the repo for configuration options.

## Usage

1. Connect your MIDI device
2. Configure the `config.toml` file (must be in CWD of the program)
3. Run the program