use tui::{backend::Backend, Frame};
use tui::layout::Rect;

use crate::{jira::tickets::TicketData, config::KeyConfig};

use super::DrawableComponent;



pub struct TicketMetadataComponent {
    // key_config: KeyConfig,
    // state: ListState,
    // ticket: TicketData,
}

impl DrawableComponent for TicketMetadataComponent {
    fn draw<B: Backend>(
        &self,
        f: &mut Frame<B>,
        _rect: Rect,
        _focused: bool,
    ) -> anyhow::Result<()> {

        Ok(())
    }
}

impl TicketMetadataComponent {
    pub fn new(key_config: KeyConfig) -> Self {

        return Self {
            // state,
            // ticket: None,
            // key_config,
        };
    }

    // pub fn next_ticket(&mut self, line: usize) {
    //     let i = match self.state.selected() {
    //         Some(i) => {
    //             if i + line >= self.ticket.len() {
    //                 Some(self.ticket.len() - 1)
    //             } else {
    //                 Some(i + line)
    //             }
    //         }
    //         None => None,
    //     };
    //
    //     self.state.select(i);
    // }
    //
    // pub fn previous_ticket(&mut self, line: usize) {
    //     let i = match self.state.selected() {
    //         Some(i) => {
    //             if i <= line {
    //                 Some(0)
    //             } else {
    //                 Some(i - line)
    //             }
    //         }
    //         None => None,
    //     };
    //
    //     self.state.select(i);
    // }
    //
    // pub fn go_to_top(&mut self) {
    //     if self.ticket.is_empty() {
    //         return;
    //     }
    //     self.state.select(Some(0));
    // }
    //
    // pub fn go_to_bottom(&mut self) {
    //     if self.ticket.is_empty() {
    //         return;
    //     }
    //     self.state.select(Some(self.ticket.len() - 1));
    // }
    //
    // pub fn selected_ticket(&self) -> Option<&TicketData> {
    //     match self.state.selected() {
    //         Some(i) => self.ticket.get(i),
    //         None => None,
    //     }
    // }
    //
    // pub async fn update(
    //     &self,
    //     db: &SurrealAny,
    //     jira_auth: &JiraAuth,
    //     project_key: &str,
    //     ticket: &JiraTickets,
    //     ) -> Result<Vec<TicketData>, SurrealDbError> {
    //     let update_ticket = ticket.get_jira_tickets(db, jira_auth, project_key).await?;
    //     Ok(update_ticket)
    // }
}
