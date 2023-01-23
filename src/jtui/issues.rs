use crossterm::terminal::enable_raw_mode;
use tui::{
    backend::CrosstermBackend,
    layout::Constraint,
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, BorderType, Borders, Cell, List, ListItem, ListState, Row, Table},
    Terminal
};
use std::{sync::mpsc, time::{Duration, Instant}, thread};
use crossterm::event::{self, Event as CEvent};
use std::io;

// enum Event<I> {
//     Input(I),
//     Tick,
// }
//
// #[derive(Copy, Clone, Debug)]
// enum MenuItem {
//     Issues,
// }
//
// impl From<MenuItem> for usize {
//     fn from(input: MenuItem) -> usize {
//         match input {
//             MenuItem::Issues => 0,
//         }
//     }
// }
//
// pub fn draw_issues() {
//     enable_raw_mode().expect("can run in raw mode");
//     let (tx, rx) = mpsc::channel();
//     let tick_rate = Duration::from_millis(200);
//     // TODO: Need to understand this better
//     thread::spawn(move || {
//         let mut last_tick = Instant::now();
//         loop {
//             let timeout = tick_rate
//                 .checked_sub(last_tick.elapsed())
//                 .unwrap_or_else(|| Duration::from_secs(0));
//
//             if event::poll(timeout).expect("poll works") {
//                 if let CEvent::Key(key) = event::read().expect("can read events") {
//                     tx.send(Event::Input(key)).expect("can send events");
//                 }
//             }
//
//             if last_tick.elapsed() >= tick_rate {
//                 if let Ok(_) = tx.send(Event::Tick) {
//                     last_tick = Instant::now();
//                 }
//             }
//         }
//     });
//
//     let stdout = io::stdout();
//     let backend = CrosstermBackend::new(stdout);
//     let mut terminal = Terminal::new(backend)?;
//     terminal.clear()?;
//
//     let mut issues_list: Vec<String> = Vec::new();
//     let mut issues_list_state = ListState::default();
//     issues_list_state.select(Some(0));
// }

pub fn render_issues<'a>(
    issue_list: &'a Vec<String>,
    issue_list_state: &ListState,
) -> (List<'a>, Table<'a>) {
    let issues_block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::White))
        .title("Issues")
        .border_type(BorderType::Plain);

    let items: Vec<_> = issue_list
        .iter()
        .map(|issue| {
            ListItem::new(Spans::from(vec![Span::styled(
                issue.to_string(),
                Style::default(),
            )]))
        })
        .collect();

    // This is some voodoo magic I need to understand better
    let selected_issue = issue_list
        .get(
            issue_list_state
                .selected()
                .expect("There should always be a selected issue"),
        )
        .expect("selected issue exists");

    let list = List::new(items).block(issues_block).highlight_style(
        Style::default()
            .bg(Color::Yellow)
            .fg(Color::Black)
            .add_modifier(Modifier::BOLD),
    );

    let table = Table::new(vec![Row::new(vec![
        Cell::from(Span::raw(selected_issue)),
        // Cell::from(Span::raw(&selected_project.key)),
        // Cell::from(Span::raw())
    ])])
    .header(Row::new(vec![
        Cell::from(Span::styled(
            "Name",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        // Cell::from(Span::styled(
        //     "Key",
        //     Style::default().add_modifier(Modifier::BOLD),
        // )),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White))
            .title("Detail")
            .border_type(BorderType::Plain),
    )
    .widths(&[
        Constraint::Percentage(5),
        Constraint::Percentage(20),
        Constraint::Percentage(20),
        Constraint::Percentage(5),
        Constraint::Percentage(20),
    ]);

    return (list, table);
}
