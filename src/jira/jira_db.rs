// use sqlite::{Connection, Result};
//
// pub fn connection() -> Connection {
//     let connection = sqlite::open(":memory:")
//         .expect("Database not created");
//
//     let query = "
//         CREATE TABLE IF NOT EXISTS projects (
//             id INTEGER PRIMARY KEY,
//             key TEXT NOT NULL,
//             name TEXT NOT NULL);
//         CREATE TABLE IF NOT EXISTS issues (
//             id INTEGER PRIMARY KEY,
//             key TEXT NO NULL,
//             summary TEXT NO NULL,
//             status TEXT NO NULL,
//             priority TEXT NO NULL,
//             issuetype TEXT NO NULL
//         );
//     ";
//
//     connection.execute(query)
//         .expect("unable to create tables");
//     return connection
// }

use surrealdb::Datastore;
use surrealdb::Error;
use surrealdb::Session;

pub async fn connection() -> Result<Datastore, Error> {
    let db = Datastore::new("memory").await?;
    Ok(db)
}
