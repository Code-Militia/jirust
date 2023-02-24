use surrealdb::engine::any::Any;
use surrealdb::Surreal;
use tui::{
    backend::Backend,
    layout::Rect,
    style::{Color, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Clear, List, ListItem, ListState},
    Frame,
};

use crate::{
    config::KeyConfig,
    event::key::Key,
    jira::{
        auth::JiraAuth,
        tickets::{JiraTickets, TicketData},
    },
};

use super::{commands::CommandInfo, Component, EventState};

type SurrealAny = Surreal<Any>;

#[derive(Debug)]
pub struct TicketWidget {
    components_state: ListState,
    key_config: KeyConfig,
    state: ListState,
    pub tickets: Vec<TicketData>,
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
            tickets
                .push(ListItem::new(vec![Spans::from(Span::raw(&i.key))]).style(Style::default()))
        }

        let ticket_list_block = List::new(tickets)
            .block(Block::default().borders(Borders::ALL).title("Tickets"))
            .highlight_style(Style::default().bg(Color::Blue))
            .style(Style::default());

        f.render_widget(Clear, rect);
        f.render_stateful_widget(ticket_list_block, rect, &mut self.state);

        Ok(())
    }

    pub fn draw_components<B: Backend>(
        &mut self,
        f: &mut Frame<B>,
        rect: Rect,
    ) -> anyhow::Result<()> {
        f.render_widget(Clear, rect);

        let ticket = match self.state.selected().and_then(|i| self.tickets.get(i)) {
            None => return Ok(()),
            Some(ticket_data) => ticket_data,
        };

        let components: Vec<_> = ticket
            .fields
            .components
            .iter()
            .map(|component| ListItem::new(component.name.as_str()))
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
        let mut state = ListState::default();
        components_state.select(Some(0));
        labels_state.select(Some(0));
        state.select(Some(0));

        return Self {
            components_state,
            key_config,
            tickets: vec![],
            state,
        };
    }

    pub fn next(&mut self, line: usize) {
        let i = self
            .state
            .selected()
            .map(|i| (i + line).min(self.tickets.len() - 1));

        self.state.select(i);
    }

    pub fn previous(&mut self, line: usize) {
        let i = self
            .state
            .selected()
            .map(|i| if i <= line { 0 } else { i - line });

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

    pub fn selected(&self) -> Option<&TicketData> {
        match self.state.selected() {
            Some(i) => self.tickets.get(i),
            None => None,
        }
    }

    pub async fn update(
        &mut self,
        db: &SurrealAny,
        jira_auth: &JiraAuth,
        project_key: &str,
        ticket: &JiraTickets,
    ) -> anyhow::Result<()> {
        self.tickets = ticket.get_jira_tickets(db, jira_auth, project_key).await?;
        Ok(())
    }
}

impl Component for TicketWidget {
    fn commands(&self, _out: &mut Vec<CommandInfo>) {}

    fn event(&mut self, key: Key) -> anyhow::Result<EventState> {
        if key == self.key_config.scroll_down {
            self.next(1);
            return Ok(EventState::Consumed);
        } else if key == self.key_config.scroll_up {
            self.previous(1);
            return Ok(EventState::Consumed);
        } else if key == self.key_config.scroll_down_multiple_lines {
            self.next(10);
            return Ok(EventState::Consumed);
        } else if key == self.key_config.scroll_up_multiple_lines {
            self.previous(10);
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
