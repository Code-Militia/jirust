use crate::config::KeyConfig;

static CMD_GROUP_GENERAL: &str = "-- General --";

#[derive(Clone, PartialEq, PartialOrd, Ord, Eq)]
pub struct CommandText {
    pub name: String,
    pub group: &'static str,
    pub hide_help: bool,
}

impl CommandText {
    pub const fn new(name: String, group: &'static str) -> Self {
        Self {
            name,
            group,
            hide_help: false,
        }
    }
}

pub struct CommandInfo {
    pub text: CommandText,
}

impl CommandInfo {
    pub const fn new(text: CommandText) -> Self {
        Self { text }
    }
}

pub fn go_back(key: &KeyConfig) -> CommandText {
    CommandText::new(format!("Go back [{}]", key.esc), CMD_GROUP_GENERAL)
}

pub fn scroll(key: &KeyConfig) -> CommandText {
    CommandText::new(
        format!(
            "Scroll up/down/left/right [{},{},{},{}]",
            key.scroll_up, key.scroll_down, key.scroll_left, key.scroll_right
        ),
        CMD_GROUP_GENERAL,
    )
}

pub fn scroll_up_down_multiple_lines(key: &KeyConfig) -> CommandText {
    CommandText::new(
        format!(
            "Scroll up/down multiple lines [{},{}]",
            key.scroll_up_multiple_lines, key.scroll_down_multiple_lines,
        ),
        CMD_GROUP_GENERAL,
    )
}

pub fn scroll_to_top_bottom(key: &KeyConfig) -> CommandText {
    CommandText::new(
        format!(
            "Scroll to top/bottom [{},{}]",
            key.scroll_to_top, key.scroll_to_bottom,
        ),
        CMD_GROUP_GENERAL,
    )
}

pub fn ticket_add_comments(key: &KeyConfig) -> CommandText {
    CommandText::new(
        format!("Add comments to ticket [{}]", key.ticket_add_comments),
        CMD_GROUP_GENERAL,
    )
}

pub fn help(key_config: &KeyConfig) -> CommandText {
    CommandText::new(
        format!("Help [{}]", key_config.open_help),
        CMD_GROUP_GENERAL,
    )
}
