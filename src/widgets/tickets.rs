use surrealdb::engine::any::Any;
use surrealdb::Surreal;
use tui::{
    backend::Backend,
    layout::Rect,
    style::{Color, Style},
    text::{Span, Spans},
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, ListState},
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

use super::{commands::CommandInfo, Component, EventState, draw_block_style, draw_highlight_style};

type SurrealAny = Surreal<Any>;

#[derive(Debug)]
pub struct TicketWidget {
    key_config: KeyConfig,
    state: ListState,
    pub tickets: Vec<TicketData>,
}

impl TicketWidget {
    pub fn draw<B: Backend>(
        &mut self,
        f: &mut Frame<B>,
        focused: bool,
        rect: Rect,
    ) -> anyhow::Result<()> {
        let title = "Tickets";
        let tckts = &self.tickets;
        let mut list_items: Vec<ListItem> = Vec::new();
        for i in tckts {
            list_items
                .push(ListItem::new(vec![Spans::from(Span::raw(&i.key))]).style(Style::default()))
        }

        let list = List::new(list_items)
            .block(draw_block_style(focused, &title))
            .highlight_style(draw_highlight_style())
            .highlight_symbol("-> ");

        f.render_widget(Clear, rect);
        f.render_stateful_widget(list, rect, &mut self.state);

        Ok(())
    }
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
