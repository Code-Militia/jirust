impl Components {
    pub fn draw<B: Backend>(
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
}
