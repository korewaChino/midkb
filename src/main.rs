// Program that takes in MIDI input from a controller
// and prints it out to the console.

use std::collections::HashMap;
mod config;
use config::Config;
use midi_msg::{ChannelVoiceMsg, ControlChange, MidiMsg};
use midir::{Ignore, MidiInput};
use mouse_keyboard_input::VirtualDevice;
use tracing::{info, trace, warn};

#[derive(Debug)]
pub enum CCDirection {
    Clockwise,
    CounterClockwise,
}

pub struct MidiInputHandler {
    device: VirtualDevice,
    config: config::Config,

    // A map for determining the direction of CC messages
    // Should contain the CC number as the key and the velocity as value, if not exists it will be created and set
    // to the last known value
    cc_map: HashMap<u8, u8>,
}

impl MidiInputHandler {
    pub fn new(device: VirtualDevice, config: Config) -> Self {
        Self {
            config,
            device,
            cc_map: HashMap::new(),
        }
    }

    fn handle_cc(&mut self, cc: ControlChange) -> CCDirection {
        let val = cc.value();
        let cc = cc.control();

        let direction = {
            if let Some(vel) = self.cc_map.get(&cc) {
                // if value is less than last known value, we are turning counter-clockwise
                // AKA, left key
                if vel < &val {
                    Some(CCDirection::Clockwise)
                } else {
                    Some(CCDirection::CounterClockwise)
                }
            } else {
                trace!(?cc, ?val, "New CC value mapped");
                self.cc_map.insert(cc, val);
                None
            }
        };

        self.cc_map.insert(cc, val);

        direction.unwrap_or(CCDirection::Clockwise)
    }

    pub fn handle_midi_msg(&mut self, msg: MidiMsg) {
        // handle ChannelVoice messages and the inner data

        if let MidiMsg::ChannelVoice { channel: _, msg } = msg {
            match msg {
                ChannelVoiceMsg::NoteOn { note, velocity: _ } => {
                    // self.device.press(KEY_H);
                    if let Some(key) = self.config.notes.get_key(note) {
                        let _ = self.device.press(key);
                    }

                    // if let Some(key) = hardcode_notes(note) {
                    //     let _ = self.device.press(key);
                    // }
                }
                ChannelVoiceMsg::NoteOff { note, velocity: _ } => {
                    // self.device.release(KEY_H);
                    if let Some(key) = self.config.notes.get_key(note) {
                        let _ = self.device.release(key);
                    }
                }

                ChannelVoiceMsg::ControlChange { control } => {
                    let direction = self.handle_cc(control);

                    trace!(?direction, "CC message handled");

                    if let Some(cc_config) = self.config.cc.get_dir_config(control.control()) {
                        trace!(?cc_config);

                        match cc_config.bind_mode {
                            config::CCBindMode::Keyboard => match direction {
                                CCDirection::CounterClockwise => {
                                    if let Some(cc_key) = cc_config.counter_clockwise.as_ref() {
                                        let _ = self.device.press(cc_key.parse().unwrap());
                                    }
                                }
                                CCDirection::Clockwise => {
                                    if let Some(cw_key) = cc_config.clockwise.as_ref() {
                                        let _ = self.device.press(cw_key.parse().unwrap());
                                    }
                                }
                            },
                            config::CCBindMode::Mouse => {
                                // only allow string of x or y inside the config
                                let speed = 10;

                                let axis = match direction {
                                    CCDirection::CounterClockwise => &cc_config.counter_clockwise,
                                    CCDirection::Clockwise => &cc_config.clockwise,
                                };

                                let (dx, dy) = match axis.as_deref() {
                                    Some("x") => (speed, 0),
                                    Some("-x") => (-speed, 0),
                                    Some("y") => (0, speed),
                                    Some("-y") => (0, -speed),
                                    _ => (0, 0),
                                };

                                let (dx, dy) = match direction {
                                    CCDirection::CounterClockwise => (-dx, -dy),
                                    CCDirection::Clockwise => (dx, dy),
                                };

                                let _ = self.device.move_mouse(dx, dy);
                            }
                            config::CCBindMode::Toggle => {
                                // Check the current velocity of the control change
                                // It should either be 0 or 127

                                // todo: probably make the velocity threshold configurable

                                let velocity = control.value();

                                if let Some(cw_key) = cc_config.clockwise.as_ref() {
                                    if velocity == 127 {
                                        let _ = self.device.press(cw_key.parse().unwrap());
                                    } else if velocity == 0 {
                                        let _ = self.device.release(cw_key.parse().unwrap());
                                    }
                                }
                            }
                        }
                    }
                }

                _ => {}
            }
        }
    }
}

fn midi_msg_callback(time: u64, midimsg: &[u8], input: &mut MidiInputHandler) {
    trace!(?time, "MIDI Message: {:02X?}", midimsg);

    // parse midi message

    let (msg, len) = match MidiMsg::from_midi(midimsg) {
        Ok(parsed) => parsed,
        Err(e) => {
            warn!(?e, "Failed to parse MIDI message");
            return;
        }
    };

    trace!(?msg, ?len, "Parsed MIDI message");

    input.handle_midi_msg(msg);
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string()))
        .init();
    tracing::info!("Starting up");
    let config_file = std::fs::read_to_string("config.toml").unwrap();
    let config: Config = toml::from_str(&config_file).unwrap();

    let mut mid_input = MidiInput::new("midir reading input").unwrap();

    mid_input.ignore(Ignore::SysexAndTime);

    let in_ports = mid_input.ports();

    tracing::info!("Available input ports:");
    for (i, p) in in_ports.iter().enumerate() {
        tracing::info!("{}: {}", i, mid_input.port_name(p).unwrap());
    }

    let in_port = match in_ports.iter().find(|p| {
        mid_input
            .port_name(p)
            .unwrap()
            .contains(config.midi_device.as_str())
    }) {
        Some(p) => p,
        None => {
            tracing::error!("No input port found");
            return;
        }
    };

    info!("Opening connection");

    let device = VirtualDevice::default().unwrap();

    let mut input_handler = MidiInputHandler::new(device, config);

    let in_port = match mid_input.connect(
        in_port,
        "midkb-bind",
        move |time, midimsg, _| midi_msg_callback(time, midimsg, &mut input_handler),
        (),
    ) {
        Ok(p) => p,
        Err(e) => {
            println!("Error: {}", e);
            return;
        }
    };

    // wait for sigint

    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            println!("Received SIGINT, exiting...");
            in_port.close();
        }
    }
}
