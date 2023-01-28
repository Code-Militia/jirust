// -- Activity trait
use surrealdb::{Datastore, Session};
use crate::jira::auth::JiraAuth;

pub trait Activity {
    /// `on_create` is the function which must be called to initialize the activity.
    /// `on_create` must initialize all the data structures used by the activity
    /// Context is taken from activity manager and will be released only when activity is destroyed
    // fn on_create(&mut self, context: Context);
    fn on_create(&mut self, auth: JiraAuth, db: &(Datastore, Session));

    /// `on_draw` is the function which draws the graphical interface.
    /// This function must be called at each tick to refresh the interface
    fn on_draw(&mut self);

    /// `will_umount` is the method which must be able to report to the activity manager, whether
    /// the activity should be terminated or not.
    /// If not, the call will return `None`, otherwise return`Some(ExitReason)`
    // fn will_umount(&self) -> Option<&ExitReason>;

    /// `on_destroy` is the function which cleans up runtime variables and data before terminating the activity.
    /// This function must be called once before terminating the activity.
    /// This function finally releases the context
    // fn on_destroy(&mut self) -> Option<Context>;
}
