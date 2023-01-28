use log::debug;
use crate::ui::activities::activities::Activity;
use std::time::Duration;

use surrealdb::{Datastore, Session};
use tuirealm::{Application, NoUserEvent, EventListenerCfg};

use crate::jira::{projects::JiraProjects, auth::JiraAuth};

#[derive(Debug, PartialEq, Hash, Clone, Eq)]
enum Id {}

#[derive(Debug, PartialEq, Hash)]
enum Msg {}

pub struct ProjectActivity {
    app: Application<Id, Msg, NoUserEvent>,
    redraw: bool
}

impl ProjectActivity {
    pub fn new(ticks: Duration) -> Self {
        Self {
            app: Application::init(
                EventListenerCfg::default()
                    .poll_timeout(ticks)
                    .default_input_listener(ticks)
            ),
            redraw: true
        }
    }
}

impl Activity for ProjectActivity {
    fn on_create(&mut self, auth: JiraAuth, db: &(Datastore, Session)) {
        debug!("Initializing activity...");
        let projects = JiraProjects {
            auth: &auth,
            db_connection: &db,
        };
        projects.save_jira_projects();
    }

    fn on_draw(&mut self) {

    }
}

// use tui::{
//     layout::Constraint,
//     style::{Color, Modifier, Style},
//     text::{Span, Spans},
//     widgets::{Block, BorderType, Borders, Cell, List, ListItem, ListState, Row, Table},
// };
//
// pub fn render_projects<'a>(
//     projects_list: &'a Vec<String>,
//     project_list_state: &ListState,
// ) -> (List<'a>, Table<'a>) {
//     let projects_block = Block::default()
//         .borders(Borders::ALL)
//         .style(Style::default().fg(Color::White))
//         .title("Projects")
//         .border_type(BorderType::Plain);
//
//     let items: Vec<_> = projects_list
//         .iter()
//         .map(|project| {
//             ListItem::new(Spans::from(vec![Span::styled(
//                 project.to_string(),
//                 Style::default(),
//             )]))
//         })
//         .collect();
//
//     // This is some voodoo magic I need to understand better
//     let selected_project = projects_list
//         .get(
//             project_list_state
//                 .selected()
//                 .expect("There should always be a selected project"),
//         )
//         .expect("selected project exists");
//
//     let list = List::new(items).block(projects_block).highlight_style(
//         Style::default()
//             .bg(Color::Yellow)
//             .fg(Color::Black)
//             .add_modifier(Modifier::BOLD),
//     );
//
//     let table = Table::new(vec![Row::new(vec![
//         Cell::from(Span::raw(selected_project)),
//         // Cell::from(Span::raw(&selected_project.key)),
//         // Cell::from(Span::raw())
//     ])])
//     .header(Row::new(vec![
//         Cell::from(Span::styled(
//             "Name",
//             Style::default().add_modifier(Modifier::BOLD),
//         )),
//         // Cell::from(Span::styled(
//         //     "Key",
//         //     Style::default().add_modifier(Modifier::BOLD),
//         // )),
//     ]))
//     .block(
//         Block::default()
//             .borders(Borders::ALL)
//             .style(Style::default().fg(Color::White))
//             .title("Detail")
//             .border_type(BorderType::Plain),
//     )
//     .widths(&[
//         Constraint::Percentage(5),
//         Constraint::Percentage(20),
//         Constraint::Percentage(20),
//         Constraint::Percentage(5),
//         Constraint::Percentage(20),
//     ]);
//
//     return (list, table);
// }
