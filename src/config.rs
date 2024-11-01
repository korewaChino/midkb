
#[derive(serde::Deserialize, Debug, Default)]
pub struct Config {
    pub cc: CCConfig,
    pub notes: NoteBinding,
    
    /// The string to search for in the midi device port
    /// e.g. "28:0" for the port containing "28:0" in the name
    pub midi_device: String,
}

#[derive(serde::Deserialize, Debug, Default)]
/// Mode to bind the CC controls to
pub enum CCBindMode {
    /// Press a key on the keyboard everytime the CC is moved
    #[default]
    Keyboard,
    /// Move the mouse everytime the CC is moved
    Mouse
}

#[derive(serde::Deserialize, Debug, Default)]
pub struct CCDirectionConfig {

    pub bind_mode: CCBindMode,

    // both counter_clockwise can be either a keycode (see keycode crate for the codes, must be a u16)
    // or a mouse axis (x, y)
    #[serde(serialize_with = "serde_with::rust::display_fromstr")]
    pub counter_clockwise: String,
    #[serde(serialize_with = "serde_with::rust::display_fromstr")]
    pub clockwise: String,
}

#[derive(serde::Deserialize, Debug, Default)]
pub struct CCConfig {
    // would be a toml of the form:
    // [cc]
    // <cc_number> = [counter_clockwise, clockwise]
    // 1 = [60, 70]
    #[serde(flatten)]
    pub cc: std::collections::HashMap<String, CCDirectionConfig>,
}
#[derive(serde::Deserialize, Debug, Default)]
pub struct NoteBinding {
    #[serde(flatten)]
    pub notes: std::collections::HashMap<String, u16>,
}

impl NoteBinding {
    pub fn get_key(&self, note: u8) -> Option<u16> {
        self.notes.get(&note.to_string()).copied()
    }
}

impl CCConfig {
    pub fn get_dir_config(&self, cc: u8) -> Option<&CCDirectionConfig> {
        self.cc.get(&cc.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_config() {
        let config = r#"
            midi_device = "28:0"
            [cc.1]
            bind_mode = "Keyboard"
            counter_clockwise = "60"
            clockwise = "70"
            [notes]
            60 = 12
        "#;

        let toml: toml::Value = toml::from_str(config).unwrap();
        println!("{:#?}", toml);
        let config: Config = toml::from_str(config).unwrap();
        println!("{:#?}", config);
    }
}
