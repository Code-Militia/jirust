use std::time::Duration;

use surrealdb::{Datastore, Session};

use crate::ui::actvities
use crate::{jira::{auth::{JiraAuth, jira_authentication}, projects::JiraProjects}, jtui::projects::ProjectActivity};

pub enum NextActivity {
    NextIssueAcitivity,
    NextProjectActivity,
}

pub type DB = (Datastore, Session);

pub struct ActivityManager {
    ticks: Duration,
    jira_auth: JiraAuth,
    db: Option<DB>,
}

impl ActivityManager {
    pub async fn new(ticks: Duration) -> Result<ActivityManager, surrealdb::Error> {
        let db: DB = (
            Datastore::new("memory").await?,
            Session::for_db("jira", "jira"),
        );
        let auth: JiraAuth = jira_authentication();

        Ok(ActivityManager {
            db: Some(db),
            jira_auth: auth,
            ticks
        })
    }

    pub fn run(&self, launch_activity: NextActivity) {
        let mut current_activity: Option<NextActivity> = Some(launch_activity);
        loop {
            current_activity = match current_activity {
                Some(activity) => match activity {
                    NextActivity::NextIssueAcitivity => {},
                    NextActivity::NextProjectActivity => self.jira_projects(),
                },
                None => break,
            }
        }
    }

    pub fn jira_projects(&self) -> Option<NextActivity> {
        let db = self.db.expect("Database session not started");
        let activity = ProjectActivity::new(self.jira_auth, &db, self.ticks);
        let result: Option<NextActivity>;
        
    }
}
