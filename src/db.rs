use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use color_eyre::{Report, eyre::eyre };
use rusqlite::{Connection, params};
use tracing::info;


use crate::command::Command;

pub struct ClockRuster {
    connection_string: String,
}

impl ClockRuster {
    pub fn new() -> Self {
        Self {
            connection_string: String::from("./.clockrust"),
        }
    }

    pub fn init(conn_str: &str) -> Self {
        Self {
            connection_string: String::from(conn_str)
        }
    }

    fn ensure_storage_exists(self, conn: &Connection) -> Result<(), Report> {
        //check for table's existence
        conn.execute("
            CREATE TABLE IF NOT EXISTS clock_rust_tasks(
                id INTEGER PRIMARY KEY ASC,
                command TEXT,
                task TEXT,
                hash INTEGER,
                cmd_date DATETIME
            )
        ", [])?;
        Ok(())
        //and create it if it does not exist
    }

    pub fn run_clock_command(self, cmd: &Command) -> Result<(), Report> {
        let conn = Connection::open(&self.connection_string)?;
        match self.ensure_storage_exists(&conn){
            Ok(_) => {
                let mut hasher = DefaultHasher::new();
                cmd.hash(&mut hasher);
                let updated = conn.execute(r"INSERT into clock_rust_tasks (command, task, hash, cmd_date)
                                    VALUES (?, ?, ?, ?);",
                             params![ cmd.command.to_string(), cmd.task, hasher.finish(), cmd.cmd_datetime  ])?;
                info!("Number of rows inserted {}", updated);
            }
            Err(y) => { return Err(eyre!("Failed to run command: {}", y))}
        }
        Ok(())
    }

}

#[cfg(test)]
mod tests{
    use color_eyre::Report;
    use tracing::{error, Level};
    use tracing_subscriber::FmtSubscriber;
    use crate::config;

    use super::*;

    const TEST_DB_STRING: &str = "./clock_rust_test";


    #[test]
    fn test_create_table(){
        config::setup_test_logging();
        let cr = ClockRuster::init(TEST_DB_STRING);
        if let Ok(conn) = Connection::open(cr.connection_string.clone()){
            match cr.ensure_storage_exists(&conn){
                Ok(_) => {info!("Successfully ran ensure_storage_exists")}
                Err(why) => {panic!("Could not ensure_storage_exists: {}", why)}
            }
            let fp = std::path::Path::new(TEST_DB_STRING);
            assert!(std::path::Path::exists(fp));
            //SELECT name FROM sqlite_master WHERE type='table' AND name='{table_name}';
            let mut stmt = conn.prepare("SELECT 1 FROM sqlite_master WHERE type='table' AND name='clock_rust_tasks'").unwrap();
            let mut rows = stmt.query([]).unwrap();
            let table_count = if let Some(row) = rows.next().unwrap(){
                 row.get_unwrap(0)
            }else{ 0 };
            //delete the file
            std::fs::remove_file("./clock_rust_test").expect("could not delete test sqlite db file");
            assert_eq!(table_count, 1)

        }else{
            panic!("Failed to get connection");
        }

    }

    // #[test]
    // fn text_run_clock_in_command(){
    //     ClockRuster::init("/")
    // }
}

