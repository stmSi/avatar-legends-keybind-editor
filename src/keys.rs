use std::collections::HashSet;

#[derive(Clone, Copy)]
struct KeyDef {
    token: &'static str,
    label: &'static str,
    vk: i32,
}

pub struct KeySnapshot {
    pub down: HashSet<String>,
    pub newly_pressed: Vec<String>,
}

pub struct KeyScanner {
    previous: HashSet<String>,
}

impl KeyScanner {
    pub fn new() -> Self {
        Self {
            previous: currently_down(),
        }
    }

    pub fn poll(&mut self) -> KeySnapshot {
        let down = currently_down();
        let newly_pressed = KEY_DEFS
            .iter()
            .filter(|key| down.contains(key.token) && !self.previous.contains(key.token))
            .map(|key| key.token.to_owned())
            .collect();
        self.previous = down.clone();
        KeySnapshot {
            down,
            newly_pressed,
        }
    }
}

pub fn key_label(token: &str) -> String {
    KEY_DEFS
        .iter()
        .find(|key| key.token == token)
        .map(|key| key.label.to_owned())
        .unwrap_or_else(|| token.trim_start_matches("KEY_").replace('_', " "))
}

#[cfg(target_os = "windows")]
fn currently_down() -> HashSet<String> {
    KEY_DEFS
        .iter()
        .filter(|key| unsafe { GetAsyncKeyState(key.vk) } < 0)
        .map(|key| key.token.to_owned())
        .collect()
}

#[cfg(not(target_os = "windows"))]
fn currently_down() -> HashSet<String> {
    HashSet::new()
}

#[cfg(target_os = "windows")]
#[link(name = "user32")]
unsafe extern "system" {
    fn GetAsyncKeyState(v_key: i32) -> i16;
}

// More specific modifier keys are deliberately listed before their generic aliases,
// so key capture prefers Left Shift over Shift while testing recognizes both.
const KEY_DEFS: &[KeyDef] = &[
    KeyDef {
        token: "KEY_ESCAPE",
        label: "Esc",
        vk: 0x1B,
    },
    KeyDef {
        token: "KEY_F1",
        label: "F1",
        vk: 0x70,
    },
    KeyDef {
        token: "KEY_F2",
        label: "F2",
        vk: 0x71,
    },
    KeyDef {
        token: "KEY_F3",
        label: "F3",
        vk: 0x72,
    },
    KeyDef {
        token: "KEY_F4",
        label: "F4",
        vk: 0x73,
    },
    KeyDef {
        token: "KEY_F5",
        label: "F5",
        vk: 0x74,
    },
    KeyDef {
        token: "KEY_F6",
        label: "F6",
        vk: 0x75,
    },
    KeyDef {
        token: "KEY_F7",
        label: "F7",
        vk: 0x76,
    },
    KeyDef {
        token: "KEY_F8",
        label: "F8",
        vk: 0x77,
    },
    KeyDef {
        token: "KEY_F9",
        label: "F9",
        vk: 0x78,
    },
    KeyDef {
        token: "KEY_F10",
        label: "F10",
        vk: 0x79,
    },
    KeyDef {
        token: "KEY_F11",
        label: "F11",
        vk: 0x7A,
    },
    KeyDef {
        token: "KEY_F12",
        label: "F12",
        vk: 0x7B,
    },
    KeyDef {
        token: "KEY_F13",
        label: "F13",
        vk: 0x7C,
    },
    KeyDef {
        token: "KEY_F14",
        label: "F14",
        vk: 0x7D,
    },
    KeyDef {
        token: "KEY_F15",
        label: "F15",
        vk: 0x7E,
    },
    KeyDef {
        token: "KEY_F16",
        label: "F16",
        vk: 0x7F,
    },
    KeyDef {
        token: "KEY_F17",
        label: "F17",
        vk: 0x80,
    },
    KeyDef {
        token: "KEY_F18",
        label: "F18",
        vk: 0x81,
    },
    KeyDef {
        token: "KEY_F19",
        label: "F19",
        vk: 0x82,
    },
    KeyDef {
        token: "KEY_F20",
        label: "F20",
        vk: 0x83,
    },
    KeyDef {
        token: "KEY_F21",
        label: "F21",
        vk: 0x84,
    },
    KeyDef {
        token: "KEY_F22",
        label: "F22",
        vk: 0x85,
    },
    KeyDef {
        token: "KEY_F23",
        label: "F23",
        vk: 0x86,
    },
    KeyDef {
        token: "KEY_F24",
        label: "F24",
        vk: 0x87,
    },
    KeyDef {
        token: "KEY_SNAPSHOT",
        label: "Print Screen",
        vk: 0x2C,
    },
    KeyDef {
        token: "KEY_SCROLLLOCK",
        label: "Scroll Lock",
        vk: 0x91,
    },
    KeyDef {
        token: "KEY_PAUSE",
        label: "Pause",
        vk: 0x13,
    },
    KeyDef {
        token: "KEY_TILDE",
        label: "`",
        vk: 0xC0,
    },
    KeyDef {
        token: "KEY_1",
        label: "1",
        vk: 0x31,
    },
    KeyDef {
        token: "KEY_2",
        label: "2",
        vk: 0x32,
    },
    KeyDef {
        token: "KEY_3",
        label: "3",
        vk: 0x33,
    },
    KeyDef {
        token: "KEY_4",
        label: "4",
        vk: 0x34,
    },
    KeyDef {
        token: "KEY_5",
        label: "5",
        vk: 0x35,
    },
    KeyDef {
        token: "KEY_6",
        label: "6",
        vk: 0x36,
    },
    KeyDef {
        token: "KEY_7",
        label: "7",
        vk: 0x37,
    },
    KeyDef {
        token: "KEY_8",
        label: "8",
        vk: 0x38,
    },
    KeyDef {
        token: "KEY_9",
        label: "9",
        vk: 0x39,
    },
    KeyDef {
        token: "KEY_0",
        label: "0",
        vk: 0x30,
    },
    KeyDef {
        token: "KEY_OEM_MINUS",
        label: "-",
        vk: 0xBD,
    },
    KeyDef {
        token: "KEY_OEM_PLUS",
        label: "=",
        vk: 0xBB,
    },
    KeyDef {
        token: "KEY_BACK",
        label: "Backspace",
        vk: 0x08,
    },
    KeyDef {
        token: "KEY_TAB",
        label: "Tab",
        vk: 0x09,
    },
    KeyDef {
        token: "KEY_Q",
        label: "Q",
        vk: 0x51,
    },
    KeyDef {
        token: "KEY_W",
        label: "W",
        vk: 0x57,
    },
    KeyDef {
        token: "KEY_E",
        label: "E",
        vk: 0x45,
    },
    KeyDef {
        token: "KEY_R",
        label: "R",
        vk: 0x52,
    },
    KeyDef {
        token: "KEY_T",
        label: "T",
        vk: 0x54,
    },
    KeyDef {
        token: "KEY_Y",
        label: "Y",
        vk: 0x59,
    },
    KeyDef {
        token: "KEY_U",
        label: "U",
        vk: 0x55,
    },
    KeyDef {
        token: "KEY_I",
        label: "I",
        vk: 0x49,
    },
    KeyDef {
        token: "KEY_O",
        label: "O",
        vk: 0x4F,
    },
    KeyDef {
        token: "KEY_P",
        label: "P",
        vk: 0x50,
    },
    KeyDef {
        token: "KEY_OEM_4",
        label: "[",
        vk: 0xDB,
    },
    KeyDef {
        token: "KEY_OEM_6",
        label: "]",
        vk: 0xDD,
    },
    KeyDef {
        token: "KEY_OEM_5",
        label: "\\",
        vk: 0xDC,
    },
    KeyDef {
        token: "KEY_CAPITAL",
        label: "Caps Lock",
        vk: 0x14,
    },
    KeyDef {
        token: "KEY_A",
        label: "A",
        vk: 0x41,
    },
    KeyDef {
        token: "KEY_S",
        label: "S",
        vk: 0x53,
    },
    KeyDef {
        token: "KEY_D",
        label: "D",
        vk: 0x44,
    },
    KeyDef {
        token: "KEY_F",
        label: "F",
        vk: 0x46,
    },
    KeyDef {
        token: "KEY_G",
        label: "G",
        vk: 0x47,
    },
    KeyDef {
        token: "KEY_H",
        label: "H",
        vk: 0x48,
    },
    KeyDef {
        token: "KEY_J",
        label: "J",
        vk: 0x4A,
    },
    KeyDef {
        token: "KEY_K",
        label: "K",
        vk: 0x4B,
    },
    KeyDef {
        token: "KEY_L",
        label: "L",
        vk: 0x4C,
    },
    KeyDef {
        token: "KEY_OEM_1",
        label: ";",
        vk: 0xBA,
    },
    KeyDef {
        token: "KEY_OEM_7",
        label: "'",
        vk: 0xDE,
    },
    KeyDef {
        token: "KEY_RETURN",
        label: "Enter",
        vk: 0x0D,
    },
    KeyDef {
        token: "KEY_LSHIFT",
        label: "Left Shift",
        vk: 0xA0,
    },
    KeyDef {
        token: "KEY_RSHIFT",
        label: "Right Shift",
        vk: 0xA1,
    },
    KeyDef {
        token: "KEY_Z",
        label: "Z",
        vk: 0x5A,
    },
    KeyDef {
        token: "KEY_X",
        label: "X",
        vk: 0x58,
    },
    KeyDef {
        token: "KEY_C",
        label: "C",
        vk: 0x43,
    },
    KeyDef {
        token: "KEY_V",
        label: "V",
        vk: 0x56,
    },
    KeyDef {
        token: "KEY_B",
        label: "B",
        vk: 0x42,
    },
    KeyDef {
        token: "KEY_N",
        label: "N",
        vk: 0x4E,
    },
    KeyDef {
        token: "KEY_M",
        label: "M",
        vk: 0x4D,
    },
    KeyDef {
        token: "KEY_OEM_COMMA",
        label: ",",
        vk: 0xBC,
    },
    KeyDef {
        token: "KEY_OEM_PERIOD",
        label: ".",
        vk: 0xBE,
    },
    KeyDef {
        token: "KEY_OEM_2",
        label: "/",
        vk: 0xBF,
    },
    KeyDef {
        token: "KEY_LCONTROL",
        label: "Left Ctrl",
        vk: 0xA2,
    },
    KeyDef {
        token: "KEY_RCONTROL",
        label: "Right Ctrl",
        vk: 0xA3,
    },
    KeyDef {
        token: "KEY_LWIN",
        label: "Left Win",
        vk: 0x5B,
    },
    KeyDef {
        token: "KEY_LMENU",
        label: "Left Alt",
        vk: 0xA4,
    },
    KeyDef {
        token: "KEY_SPACE",
        label: "Space",
        vk: 0x20,
    },
    KeyDef {
        token: "KEY_RMENU",
        label: "Right Alt",
        vk: 0xA5,
    },
    KeyDef {
        token: "KEY_RWIN",
        label: "Right Win",
        vk: 0x5C,
    },
    KeyDef {
        token: "KEY_APPS",
        label: "Menu",
        vk: 0x5D,
    },
    KeyDef {
        token: "KEY_INSERT",
        label: "Insert",
        vk: 0x2D,
    },
    KeyDef {
        token: "KEY_HOME",
        label: "Home",
        vk: 0x24,
    },
    KeyDef {
        token: "KEY_PRIOR",
        label: "Page Up",
        vk: 0x21,
    },
    KeyDef {
        token: "KEY_DELETE",
        label: "Delete",
        vk: 0x2E,
    },
    KeyDef {
        token: "KEY_END",
        label: "End",
        vk: 0x23,
    },
    KeyDef {
        token: "KEY_NEXT",
        label: "Page Down",
        vk: 0x22,
    },
    KeyDef {
        token: "KEY_UP",
        label: "Up Arrow",
        vk: 0x26,
    },
    KeyDef {
        token: "KEY_LEFT",
        label: "Left Arrow",
        vk: 0x25,
    },
    KeyDef {
        token: "KEY_DOWN",
        label: "Down Arrow",
        vk: 0x28,
    },
    KeyDef {
        token: "KEY_RIGHT",
        label: "Right Arrow",
        vk: 0x27,
    },
    KeyDef {
        token: "KEY_NUMLOCK",
        label: "Num Lock",
        vk: 0x90,
    },
    KeyDef {
        token: "KEY_DIVIDE",
        label: "Num /",
        vk: 0x6F,
    },
    KeyDef {
        token: "KEY_MULTIPLY",
        label: "Num *",
        vk: 0x6A,
    },
    KeyDef {
        token: "KEY_SUBTRACT",
        label: "Num -",
        vk: 0x6D,
    },
    KeyDef {
        token: "KEY_NUMPAD7",
        label: "Num 7",
        vk: 0x67,
    },
    KeyDef {
        token: "KEY_NUMPAD8",
        label: "Num 8",
        vk: 0x68,
    },
    KeyDef {
        token: "KEY_NUMPAD9",
        label: "Num 9",
        vk: 0x69,
    },
    KeyDef {
        token: "KEY_ADD",
        label: "Num +",
        vk: 0x6B,
    },
    KeyDef {
        token: "KEY_NUMPAD4",
        label: "Num 4",
        vk: 0x64,
    },
    KeyDef {
        token: "KEY_NUMPAD5",
        label: "Num 5",
        vk: 0x65,
    },
    KeyDef {
        token: "KEY_NUMPAD6",
        label: "Num 6",
        vk: 0x66,
    },
    KeyDef {
        token: "KEY_NUMPAD1",
        label: "Num 1",
        vk: 0x61,
    },
    KeyDef {
        token: "KEY_NUMPAD2",
        label: "Num 2",
        vk: 0x62,
    },
    KeyDef {
        token: "KEY_NUMPAD3",
        label: "Num 3",
        vk: 0x63,
    },
    KeyDef {
        token: "KEY_NUMPAD0",
        label: "Num 0",
        vk: 0x60,
    },
    KeyDef {
        token: "KEY_DECIMAL",
        label: "Num .",
        vk: 0x6E,
    },
    KeyDef {
        token: "KEY_SEPARATOR",
        label: "Num Separator",
        vk: 0x6C,
    },
    KeyDef {
        token: "KEY_SHIFT",
        label: "Shift",
        vk: 0x10,
    },
    KeyDef {
        token: "KEY_CONTROL",
        label: "Ctrl",
        vk: 0x11,
    },
    KeyDef {
        token: "KEY_MENU",
        label: "Alt",
        vk: 0x12,
    },
    KeyDef {
        token: "KEY_PRINT",
        label: "Print",
        vk: 0x2A,
    },
];
