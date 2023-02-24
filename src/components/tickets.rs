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
pub struct TicketWidget {
    components_state: ListState,
    key_config: KeyConfig,
    labels_state: ListState,
    tickets_state: ListState,
    tickets: Vec<TicketData>,
}

impl TicketWidget {
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
        f.render_stateful_widget(ticket_list_block, rect, &mut self.tickets_state);

        Ok(())
    }

    pub fn draw_labels<B: Backend>(&mut self, f: &mut Frame<B>, rect: Rect) -> anyhow::Result<()> {
        f.render_widget(Clear, rect);

        let ticket = match self.tickets_state.selected().and_then(|i| self.tickets.get(i)) {
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

        f.render_stateful_widget(labels_block, rect, &mut self.labels_state);

        Ok(()) 
    }

    pub fn draw_components<B: Backend>(&mut self, f: &mut Frame<B>, rect: Rect) -> anyhow::Result<()> {
        f.render_widget(Clear, rect);

        let ticket = match self.tickets_state.selected().and_then(|i| self.tickets.get(i)) {
            None => return Ok(()),
            Some(ticket_data) => ticket_data
        };

        let components: Vec<_> = ticket.fields.components.iter()
            .map(|component| {
                ListItem::new(component.name.as_str())
            })
            .collect();

        let components_block = List::new(components)
            .block(Block::default().borders(Borders::ALL).title("Components"))
            .highlight_style(Style::default().bg(Color::Blue));

        f.render_stateful_widget(components_block, rect, &mut self.components_state);

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


impl TicketWidget {
    pub fn new(key_config: KeyConfig) -> Self {
        let mut components_state = ListState::default();
        let mut labels_state = ListState::default();
        let mut tickets_state = ListState::default();
        components_state.select(Some(0));
        labels_state.select(Some(0));
        tickets_state.select(Some(0));

        return Self {
            components_state,
            key_config,
            labels_state,
            tickets: vec![],
            tickets_state,
        };
    }

    pub fn next_ticket(&mut self, line: usize) {
        let i = self.tickets_state.selected().map(|i| {
            (i + line).min(self.tickets.len() - 1)
        });

        self.tickets_state.select(i);
    }

    // pub fn next_label(&mut self, line: usize) {
    //     let labels_len = self.selected_ticket().map(|t| t.fields.labels.len()).unwrap_or(0);
    //     let i = self.labels_state.selected().map(|_i| {
    //         line.min(labels_len - 1)
    //     });
    //
    //     self.labels_state.select(i);
    // }

    pub fn next_label(&mut self, line: usize) {
        let labels_len = self.selected_ticket().map(|t| t.fields.labels.len()).unwrap_or(0);
        let i = self.labels_state.selected().map(|i| {
            (i + line).min(labels_len - 1)
        });
        self.labels_state.select(i);
    }

    pub fn previous_ticket(&mut self, line: usize) {
        let i = self.tickets_state.selected().map(|i| {
            (i - line).min(self.tickets.len() - 1)
        });

        self.tickets_state.select(i);
    }

    pub fn previous_label(&mut self, line: usize) {
        let labels_len = self.selected_ticket().map(|t| t.fields.labels.len()).unwrap_or(0);
        let i = self.labels_state.selected().map(|i| {
            (i - line).min(labels_len - 1)
        });
        self.labels_state.select(i);
    }

    pub fn label_go_to_top(&mut self) {
        let ticket = self.selected_ticket().unwrap();
        if ticket.fields.labels.is_empty() {
            return;
        }
        self.labels_state.select(Some(0))
    }

    pub fn ticket_go_to_top(&mut self) {
        if self.tickets.is_empty() {
            return;
        }
        self.tickets_state.select(Some(0));
    }

    pub fn ticket_go_to_bottom(&mut self) {
        if self.tickets.is_empty() {
            return;
        }
        self.tickets_state.select(Some(self.tickets.len() - 1));
    }

    pub fn label_go_to_bottom(&mut self) {
        let ticket = self.selected_ticket().unwrap();
        if ticket.fields.labels.is_empty() {
            return;
        }
        self.labels_state.select(Some(ticket.fields.labels.len() - 1))
    }

    pub fn selected_ticket(&self) -> Option<&TicketData> {
        match self.tickets_state.selected() {
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

impl TicketWidget {
    pub fn label_event(&mut self, key: Key) -> anyhow::Result<EventState> {
        if key == self.key_config.scroll_down {
            self.next_label(1);
            return Ok(EventState::Consumed)
        } else if key == self.key_config.scroll_up {
            self.previous_label(1);
            return Ok(EventState::Consumed)
        } else if key == self.key_config.scroll_down_multiple_lines {
            self.next_label(10);
            return Ok(EventState::Consumed);
        } else if key == self.key_config.scroll_up_multiple_lines {
            self.previous_label(10);
            return Ok(EventState::Consumed);
        } else if key == self.key_config.scroll_to_bottom {
            self.label_go_to_bottom();
            return Ok(EventState::Consumed);
        } else if key == self.key_config.scroll_to_top {
            self.label_go_to_top();
            return Ok(EventState::Consumed);
        }
        return Ok(EventState::NotConsumed)
    }
}

impl Component for TicketWidget {
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
            self.ticket_go_to_bottom();
            return Ok(EventState::Consumed);
        } else if key == self.key_config.scroll_to_top {
            self.ticket_go_to_top();
            return Ok(EventState::Consumed);
        }
        return Ok(EventState::NotConsumed);
    }
}
