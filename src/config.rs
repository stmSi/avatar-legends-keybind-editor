use std::collections::{HashMap, HashSet};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug)]
pub struct Binding {
    pub line_index: usize,
    pub player: u8,
    pub device: u8,
    pub source: String,
    pub action: String,
}

impl Binding {
    pub fn is_keyboard(&self) -> bool {
        self.source.starts_with("KEY_")
    }
}

#[derive(Clone, Debug)]
pub struct ConflictGroup {
    pub key: String,
    pub members: Vec<Binding>,
}

#[derive(Clone, Debug)]
pub struct MacroDefinition {
    pub player: u8,
    pub index: u8,
    pub inputs: Vec<String>,
}

#[derive(Debug)]
pub struct ButtonConfig {
    lines: Vec<String>,
    newline: String,
    trailing_newline: bool,
    pub bindings: Vec<Binding>,
    pub macros: Vec<MacroDefinition>,
    pub path: PathBuf,
    pub dirty: bool,
}

impl ButtonConfig {
    pub fn load(path: impl AsRef<Path>) -> io::Result<Self> {
        let path = path.as_ref().to_path_buf();
        let text = fs::read_to_string(&path)?;
        let newline = if text.contains("\r\n") { "\r\n" } else { "\n" }.to_owned();
        let trailing_newline = text.ends_with('\n');
        let normalized = text.replace("\r\n", "\n");
        let lines = normalized.lines().map(str::to_owned).collect::<Vec<_>>();
        let mut document = Self {
            lines,
            newline,
            trailing_newline,
            bindings: Vec::new(),
            macros: Vec::new(),
            path,
            dirty: false,
        };
        document.reparse();
        Ok(document)
    }

    pub fn save(&mut self) -> io::Result<PathBuf> {
        let backup_path = self.create_backup()?;

        let mut output = self.lines.join(&self.newline);
        if self.trailing_newline {
            output.push_str(&self.newline);
        }
        fs::write(&self.path, output)?;
        self.dirty = false;
        Ok(backup_path)
    }

    pub fn create_backup(&self) -> io::Result<PathBuf> {
        if !self.path.exists() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "the active config does not exist yet",
            ));
        }

        let backup_dir = self.backup_dir();
        fs::create_dir_all(&backup_dir)?;
        let stem = self
            .path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or("button_config");
        let extension = self
            .path
            .extension()
            .and_then(|extension| extension.to_str())
            .unwrap_or("ini");
        let backup_path = (1_u32..)
            .map(|number| backup_dir.join(format!("{stem}.backup-{number:03}.{extension}")))
            .find(|candidate| !candidate.exists())
            .expect("the backup number range is effectively unlimited");
        fs::copy(&self.path, &backup_path)?;
        Ok(backup_path)
    }

    pub fn restore_from(&mut self, backup_path: impl AsRef<Path>) -> io::Result<()> {
        let text = fs::read_to_string(backup_path)?;
        self.newline = if text.contains("\r\n") { "\r\n" } else { "\n" }.to_owned();
        self.trailing_newline = text.ends_with('\n');
        self.lines = text
            .replace("\r\n", "\n")
            .lines()
            .map(str::to_owned)
            .collect();
        self.dirty = true;
        self.reparse();
        Ok(())
    }

    pub fn backup_dir(&self) -> PathBuf {
        self.path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .join("button_config_backups")
    }

    pub fn keyboard_bindings(&self, player: u8) -> Vec<Binding> {
        self.bindings
            .iter()
            .filter(|binding| binding.player == player && binding.is_keyboard())
            .cloned()
            .collect()
    }

    pub fn set_source(&mut self, line_index: usize, source: &str) -> bool {
        let Some(binding) = self
            .bindings
            .iter()
            .find(|binding| binding.line_index == line_index)
            .cloned()
        else {
            return false;
        };

        self.lines[line_index] =
            format!("Device {} {} -> {}", binding.device, source, binding.action);
        self.dirty = true;
        self.reparse();
        true
    }

    pub fn remove_binding(&mut self, line_index: usize) -> bool {
        if line_index >= self.lines.len()
            || !self
                .bindings
                .iter()
                .any(|binding| binding.line_index == line_index)
        {
            return false;
        }
        self.lines.remove(line_index);
        self.dirty = true;
        self.reparse();
        true
    }

    pub fn add_keyboard_binding(&mut self, player: u8, action: &str, source: &str) -> bool {
        let Some((section_start, section_end)) = self.player_section(player) else {
            return false;
        };

        let device = self
            .bindings
            .iter()
            .find(|binding| binding.player == player && binding.is_keyboard())
            .map(|binding| binding.device)
            .unwrap_or(6 + player);

        let insert_at = self
            .bindings
            .iter()
            .filter(|binding| binding.player == player && binding.is_keyboard())
            .map(|binding| binding.line_index + 1)
            .max()
            .unwrap_or_else(|| {
                (section_start..section_end)
                    .rev()
                    .find(|index| self.lines[*index].starts_with("Device "))
                    .map(|index| index + 1)
                    .unwrap_or(section_end)
            });

        self.lines
            .insert(insert_at, format!("Device {device} {source} -> {action}"));
        self.dirty = true;
        self.reparse();
        true
    }

    pub fn conflict_groups(&self) -> Vec<ConflictGroup> {
        let mut grouped: HashMap<String, Vec<Binding>> = HashMap::new();

        for binding in self.bindings.iter().filter(|binding| binding.is_keyboard()) {
            if binding.source == "KEY_NONE" {
                continue;
            }
            grouped
                .entry(binding.source.clone())
                .or_default()
                .push(binding.clone());
        }

        let mut conflicts = grouped
            .into_iter()
            .filter(|(_, members)| members.len() > 1)
            .map(|(key, members)| ConflictGroup { key, members })
            .collect::<Vec<_>>();
        conflicts.sort_by(|left, right| left.key.cmp(&right.key));
        conflicts
    }

    pub fn active_actions(&self, player: u8, down: &HashSet<String>) -> HashSet<String> {
        let mut active = HashSet::new();

        for binding in self.bindings.iter().filter(|binding| {
            binding.player == player && binding.is_keyboard() && down.contains(&binding.source)
        }) {
            active.insert(binding.action.clone());
            if let Some(macro_index) = macro_index_for_action(&binding.action)
                && let Some(definition) = self.macros.iter().find(|definition| {
                    definition.player == player && definition.index == macro_index
                })
            {
                for input in &definition.inputs {
                    if let Some(action) = normalize_macro_input(input) {
                        active.insert(action);
                    }
                }
            }
        }

        active
    }

    pub fn macro_inputs_for_action(&self, player: u8, action: &str) -> Option<Vec<String>> {
        let macro_index = macro_index_for_action(action)?;
        self.macros
            .iter()
            .find(|definition| definition.player == player && definition.index == macro_index)
            .map(|definition| {
                definition
                    .inputs
                    .iter()
                    .filter_map(|input| normalize_macro_input(input))
                    .collect()
            })
    }

    fn reparse(&mut self) {
        self.bindings.clear();
        self.macros.clear();
        let mut current_player = None;

        for (line_index, line) in self.lines.iter().enumerate() {
            let trimmed = line.trim();
            if let Some(number) = trimmed.strip_prefix("Player ") {
                current_player = number.parse::<u8>().ok();
                continue;
            }

            let Some(player) = current_player else {
                continue;
            };
            let pieces = trimmed.split_whitespace().collect::<Vec<_>>();
            if pieces.len() == 5
                && pieces[0] == "Device"
                && pieces[3] == "->"
                && let Ok(device) = pieces[1].parse::<u8>()
            {
                self.bindings.push(Binding {
                    line_index,
                    player,
                    device,
                    source: pieces[2].to_owned(),
                    action: pieces[4].to_owned(),
                });
            } else if pieces.len() == 4
                && pieces[0] == "Macro"
                && pieces[2] == "->"
                && let Ok(index) = pieces[1].parse::<u8>()
            {
                self.macros.push(MacroDefinition {
                    player,
                    index,
                    inputs: pieces[3].split('+').map(str::to_owned).collect(),
                });
            }
        }
    }

    fn player_section(&self, player: u8) -> Option<(usize, usize)> {
        let marker = format!("Player {player}");
        let start = self.lines.iter().position(|line| line.trim() == marker)?;
        let end = self
            .lines
            .iter()
            .enumerate()
            .skip(start + 1)
            .find(|(_, line)| line.trim().starts_with("Player "))
            .map(|(index, _)| index)
            .unwrap_or(self.lines.len());
        Some((start + 1, end))
    }
}

fn macro_index_for_action(action: &str) -> Option<u8> {
    match action {
        "Atk6" => Some(0),
        "Atk8" => Some(1),
        "Atk9" => Some(2),
        "Atk10" => Some(3),
        _ => None,
    }
}

fn normalize_macro_input(input: &str) -> Option<String> {
    match input {
        "N" | "Neutral" => None,
        "U" | "Up" => Some("Up".to_owned()),
        "D" | "Down" => Some("Down".to_owned()),
        "L" | "Left" => Some("Left".to_owned()),
        "R" | "Right" => Some("Right".to_owned()),
        action if action.starts_with("Atk") => Some(action.to_owned()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn sample() -> ButtonConfig {
        let mut config = ButtonConfig {
            lines: vec![
                "Player 1".into(),
                "Device 7 KEY_J -> Atk1".into(),
                "Device 7 KEY_SPACE -> Up".into(),
                "Player 2".into(),
                "Device 8 KEY_J -> Left".into(),
            ],
            newline: "\n".into(),
            trailing_newline: true,
            bindings: vec![],
            macros: vec![],
            path: PathBuf::from("test.ini"),
            dirty: false,
        };
        config.reparse();
        config
    }

    #[test]
    fn detects_cross_player_keyboard_conflicts() {
        let config = sample();
        let conflicts = config.conflict_groups();
        assert_eq!(conflicts.len(), 1);
        assert_eq!(conflicts[0].key, "KEY_J");
        assert_eq!(conflicts[0].members.len(), 2);
    }

    #[test]
    fn edits_and_adds_bindings() {
        let mut config = sample();
        assert!(config.set_source(1, "KEY_K"));
        assert!(config.add_keyboard_binding(2, "Right", "KEY_NUMPAD6"));
        assert!(config.bindings.iter().any(|binding| {
            binding.player == 2 && binding.action == "Right" && binding.source == "KEY_NUMPAD6"
        }));
    }

    #[test]
    fn expands_macro_buttons_into_primary_actions() {
        let mut config = sample();
        assert!(config.add_keyboard_binding(1, "Atk6", "KEY_M"));
        let player_two_marker = config
            .lines
            .iter()
            .position(|line| line == "Player 2")
            .unwrap();
        config
            .lines
            .insert(player_two_marker, "Macro 0 -> N+Atk2+Atk3".into());
        config.reparse();

        let down = HashSet::from(["KEY_M".to_owned()]);
        let active = config.active_actions(1, &down);
        assert!(active.contains("Atk6"));
        assert!(active.contains("Atk2"));
        assert!(active.contains("Atk3"));
    }

    #[test]
    fn creates_numbered_backups_and_restores_them() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let directory = std::env::temp_dir().join(format!(
            "avatar-bind-studio-{}-{unique}",
            std::process::id()
        ));
        fs::create_dir_all(&directory).unwrap();
        let path = directory.join("button_config.ini");
        fs::write(
            &path,
            "Player 1\r\nDevice 7 KEY_J -> Atk1\r\nPlayer 2\r\nDevice 8 KEY_NUMPAD7 -> Atk1\r\n",
        )
        .unwrap();

        let mut config = ButtonConfig::load(&path).unwrap();
        let first = config.create_backup().unwrap();
        assert!(first.ends_with("button_config.backup-001.ini"));

        let line = config
            .bindings
            .iter()
            .find(|binding| binding.player == 1)
            .unwrap()
            .line_index;
        assert!(config.set_source(line, "KEY_K"));
        let second = config.save().unwrap();
        assert!(second.ends_with("button_config.backup-002.ini"));
        assert!(fs::read_to_string(&path).unwrap().contains("KEY_K"));

        config.restore_from(&first).unwrap();
        assert!(
            config
                .bindings
                .iter()
                .any(|binding| binding.source == "KEY_J")
        );
        assert!(config.dirty);

        fs::remove_dir_all(directory).unwrap();
    }
}
