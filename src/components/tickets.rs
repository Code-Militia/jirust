use log::info;
use surrealdb::Surreal;
use surrealdb::engine::any::Any;
use tui::{
    backend::Backend,
    layout::Rect,
    style::{Color, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, List, ListItem, ListState, Clear},
    Frame,
};

use crate::app::LoadState;
use crate::{config::KeyConfig, event::key::Key, jira::{tickets::{TicketData, JiraTickets}, auth::JiraAuth}};

// use super::DrawableComponent;
// use super::StatefulDrawableComponent;
use super::{commands::CommandInfo, Component, EventState};

type SurrealAny = Surreal<Any>;

#[derive(Debug)]
pub struct TicketComponent {
    key_config: KeyConfig,
    state: ListState,
    tickets: Vec<TicketData>,
}

impl TicketComponent {
    pub fn draw<B: Backend>(
        &mut self,
        f: &mut Frame<B>,
        rect: Rect,
        _focused: bool,
    ) -> anyhow::Result<()> {
        let tckts = &self.tickets;
        let mut tickets: Vec<ListItem> = Vec::new();
        for i in tckts {
            tickets.push(ListItem::new(vec![Spans::from(Span::raw(&i.key))]).style(Style::default()))
        }

        let ticket_list_block = List::new(tickets)
            .block(Block::default().borders(Borders::ALL).title("Tickets"))
            .highlight_style(Style::default().bg(Color::Blue))
            .style(Style::default());

        f.render_widget(Clear, rect);
        f.render_stateful_widget(ticket_list_block, rect, &mut self.state);

        Ok(())
    }

    pub fn draw_metadata<B: Backend>(&mut self, f: &mut Frame<B>, rect: Rect) -> anyhow::Result<()> {
        f.render_widget(Clear, rect);

        let ticket = match self.state.selected().and_then(|i| self.tickets.get(i)) {
            None => return Ok(()),
            Some(ticket_data) => ticket_data
        };

        let labels: Vec<_> = ticket.fields.labels.iter()
            .map(|label| {
                ListItem::new(label.as_str())
            })
            .collect();

        let labels_block = List::new(labels)
            .block(Block::default().borders(Borders::ALL).title("Labels"))
            .highlight_style(Style::default().bg(Color::Blue));

        f.render_stateful_widget(labels_block, rect, &mut self.state);

        Ok(()) 
    }
    //
    // fn draw_work_log<B: Backend>(&self, f: &mut Frame<B>, _rect: Rect) -> anyhow::Result<()> {
    //     Ok(())
    // }
    //
    // fn draw_description<B: Backend>(&self, f: &mut Frame<B>, _rect: Rect) -> anyhow::Result<()> {
    //     Ok(()) 
    // }

}


impl TicketComponent {
    pub fn new(key_config: KeyConfig) -> Self {
        let mut state = ListState::default();
        state.select(Some(0));

        return Self {
            state,
            tickets: vec![],
            key_config,
        };
    }

    pub fn next_ticket(&mut self, line: usize) {
        let i = match self.state.selected() {
            Some(i) => {
                if i + line >= self.tickets.len() {
                    Some(self.tickets.len() - 1)
                } else {
                    Some(i + line)
                }
            }
            None => None,
        };

        self.state.select(i);
    }

    pub fn previous_ticket(&mut self, line: usize) {
        let i = match self.state.selected() {
            Some(i) => {
                if i <= line {
                    Some(0)
                } else {
                    Some(i - line)
                }
            }
            None => None,
        };

        self.state.select(i);
    }

    pub fn go_to_top(&mut self) {
        if self.tickets.is_empty() {
            return;
        }
        self.state.select(Some(0));
    }

    pub fn go_to_bottom(&mut self) {
        if self.tickets.is_empty() {
            return;
        }
        self.state.select(Some(self.tickets.len() - 1));
    }

    pub fn selected_ticket(&self) -> Option<&TicketData> {
        match self.state.selected() {
            Some(i) => self.tickets.get(i),
            None => None,
        }
    }

    pub async fn update(
        &mut self,
        db: &SurrealAny,
        jira_auth: &JiraAuth,
        load_state: &mut LoadState,
        project_key: &str,
        ticket: &JiraTickets,
        ) -> anyhow::Result<()> {
        // Call self.load_state = LoadState::loading
        *load_state = LoadState::Loading;
        self.tickets = ticket.get_jira_tickets(db, jira_auth, project_key).await?;
        // Chekc to see if there is a return from JIRA or DB and change state to complete
        info!("Calling tickets update componeent {:?}", self.tickets);
        Ok(())
    }
}

impl Component for TicketComponent {
    fn commands(&self, _out: &mut Vec<CommandInfo>) {}

    fn event(&mut self, key: Key) -> anyhow::Result<EventState> {
        if key == self.key_config.scroll_down {
            self.next_ticket(1);
            return Ok(EventState::Consumed);
        } else if key == self.key_config.scroll_up {
            self.previous_ticket(1);
            return Ok(EventState::Consumed);
        } else if key == self.key_config.scroll_down_multiple_lines {
            self.next_ticket(10);
            return Ok(EventState::Consumed);
        } else if key == self.key_config.scroll_up_multiple_lines {
            self.previous_ticket(10);
            return Ok(EventState::Consumed);
        } else if key == self.key_config.scroll_to_bottom {
            self.go_to_bottom();
            return Ok(EventState::Consumed);
        } else if key == self.key_config.scroll_to_top {
            self.go_to_top();
            return Ok(EventState::Consumed);
        }
        return Ok(EventState::NotConsumed);
    }
}
