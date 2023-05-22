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

pub fn filter(key: &KeyConfig) -> CommandText {
    CommandText::new(format!("Filter [{}]", key.filter), CMD_GROUP_GENERAL)
}

pub fn move_focus(key: &KeyConfig) -> CommandText {
    CommandText::new(
        format!(
            "Move focus to left/right [{},{}] up/down [{}, {}]",
            key.move_right, key.move_left, key.move_up, key.move_down
        ),
        CMD_GROUP_GENERAL,
    )
}

pub fn move_focus_with_tab(key: &KeyConfig) -> CommandText {
    CommandText::new(
        format!("Move focus [{}, {}]", key.next, "<S-Tab>"),
        CMD_GROUP_GENERAL,
    )
}

pub fn ticket_add_comments(key: &KeyConfig) -> CommandText {
    CommandText::new(
        format!("Add comments to ticket [{}]", key.ticket_add_comments),
        CMD_GROUP_GENERAL,
    )
}

pub fn ticket_open_browser(key: &KeyConfig) -> CommandText {
    CommandText::new(
        format!("View ticket in browser [{}]", key.open_browser),
        CMD_GROUP_GENERAL,
    )
}

pub fn ticket_view_comments(key: &KeyConfig) -> CommandText {
    CommandText::new(
        format!("View comments on ticket [{}]", key.ticket_view_comments),
        CMD_GROUP_GENERAL,
    )
}

pub fn tickets_reset(key: &KeyConfig) -> CommandText {
    CommandText::new(
        format!(
            "Clear out tickets cache table and pull from Jira [{}]",
            key.reset
        ),
        CMD_GROUP_GENERAL,
    )
}

pub fn projects_reset(key: &KeyConfig) -> CommandText {
    CommandText::new(
        format!(
            "Clear out projects cache table and pull from Jira [{}]",
            key.reset
        ),
        CMD_GROUP_GENERAL,
    )
}

pub fn ticket_transition(key: &KeyConfig) -> CommandText {
    CommandText::new(
        format!(
            "Transition ticket to another status [{}]",
            key.ticket_transition
        ),
        CMD_GROUP_GENERAL,
    )
}

pub fn help(key_config: &KeyConfig) -> CommandText {
    CommandText::new(
        format!("Help [{}]", key_config.open_help),
        CMD_GROUP_GENERAL,
    )
}

pub fn exit_pop_up(key_config: &KeyConfig) -> CommandText {
    CommandText::new(
        format!("Exit pop up [{}]", key_config.esc),
        CMD_GROUP_GENERAL,
    )
}
