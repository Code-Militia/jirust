// use crate::{event::key::Key, log::LogLevel};
use crate::event::key::Key;

use serde::Deserialize;

#[cfg(test)] // TODO: What does this do?
use serde::Serialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    #[serde(default)]
    pub key_config: KeyConfig,
    // #[serde(default)]
    // pub log_level: LogLevel,
}

#[derive(Debug, Deserialize, Clone)]
#[cfg_attr(test, derive(Serialize))]
pub struct KeyConfig {
    pub scroll_up: Key,
    pub scroll_down: Key,
    pub scroll_right: Key,
    pub scroll_left: Key,
    pub move_up: Key,
    pub move_down: Key,
    pub move_left: Key,
    pub move_right: Key,
    pub copy: Key,
    pub enter: Key,
    pub exit: Key,
    pub quit: Key,
    pub edit_mode: Key,
    pub esc: Key,
    pub focus_add_comments: Key,
    pub focus_right: Key,
    pub focus_left: Key,
    pub focus_above: Key,
    pub focus_comments: Key,
    pub focus_below: Key,
    pub focus_projects: Key,
    pub open_help: Key,
    pub filter: Key,
    pub move_ticket: Key,
    pub next_page: Key,
    pub previous_page: Key,
    pub scroll_down_multiple_lines: Key,
    pub scroll_up_multiple_lines: Key,
    pub scroll_to_top: Key,
    pub scroll_to_bottom: Key,
    pub ticket_status: Key,
    pub extend_selection_by_one_cell_left: Key,
    pub extend_selection_by_one_cell_right: Key,
    pub extend_selection_by_one_cell_up: Key,
    pub extend_selection_by_one_cell_down: Key,
    pub extend_or_shorten_widget_width_to_right: Key,
    pub extend_or_shorten_widget_width_to_left: Key,
}

impl Default for KeyConfig {
    fn default() -> Self {
        Self {
            scroll_up: Key::Char('k'),
            scroll_down: Key::Char('j'),
            scroll_right: Key::Char('l'),
            scroll_left: Key::Char('h'),
            move_up: Key::Up,
            move_down: Key::Down,
            move_left: Key::Left,
            move_right: Key::Right,
            copy: Key::Char('y'),
            enter: Key::Enter,
            exit: Key::Ctrl('c'),
            quit: Key::Char('q'),
            edit_mode: Key::Char('e'),
            esc: Key::Esc,
            focus_add_comments: Key::Char('C'),
            focus_right: Key::Right,
            focus_left: Key::Left,
            focus_above: Key::Up,
            focus_comments: Key::Char('c'),
            focus_below: Key::Down,
            focus_projects: Key::Char('p'),
            open_help: Key::Char('?'),
            filter: Key::Char('/'),
            move_ticket: Key::Enter,
            next_page: Key::Char('n'),
            previous_page: Key::Char('N'),
            scroll_down_multiple_lines: Key::Ctrl('d'),
            scroll_up_multiple_lines: Key::Ctrl('u'),
            scroll_to_top: Key::Char('g'),
            scroll_to_bottom: Key::Char('G'),
            ticket_status: Key::Char('s'),
            extend_selection_by_one_cell_left: Key::Char('H'),
            extend_selection_by_one_cell_right: Key::Char('L'),
            extend_selection_by_one_cell_down: Key::Char('J'),
            extend_selection_by_one_cell_up: Key::Char('K'),
            extend_or_shorten_widget_width_to_right: Key::Char('>'),
            extend_or_shorten_widget_width_to_left: Key::Char('<'),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            key_config: KeyConfig::default(),
        }
    }
}

impl Config {
    pub fn new() -> anyhow::Result<Self> {
        Ok(Config::default())
    }
}
