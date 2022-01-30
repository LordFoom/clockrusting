use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use chrono::{DateTime, Utc};
use color_eyre::{Report, eyre::eyre};
use rusqlite::{Connection, params};
use tracing::info;
// use std::str::FromStr;


use crate::command::{Command, CommandType};

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

    fn ensure_storage_exists(&self, conn: &Connection) -> Result<(), Report> {
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

    pub fn run_clock_command(&self, cmd: &Command) -> Result<(), Report> {
        let conn = Connection::open(&self.connection_string)?;
        match self.ensure_storage_exists(&conn){
            Ok(_) => {
                let mut hasher = DefaultHasher::new();
                cmd.hash(&mut hasher);
                let updated = conn.execute(r"INSERT into clock_rust_tasks (command, task, hash, cmd_date)
                                    VALUES (?, ?, ?, ?);",
                             params![ cmd.command.to_string(), cmd.task, hasher.finish() as i64, cmd.cmd_datetime  ])?;
                info!("Number of rows inserted {}", updated);
            }
            Err(y) => { return Err(eyre!("Failed to run command: {}", y))}
        }
        Ok(())
    }

    ///Simplistic "are we tracking this task?" method
    /// We count the number of clock-in commands
    /// If > 0, count clock-out commands
    /// iff clock-in count > clock-out count, return true
    /// Else return false
    pub fn currently_tracking(&self, task:&str)->Result<bool, Report>{
        let conn = Connection::open(&self.connection_string)?;
        self.ensure_storage_exists(&conn)?;
        let mut hasher = DefaultHasher::new();
        task.hash(&mut hasher);
        let hash = hasher.finish() as i64;
        //get number clock-in commands
        let cic = self.count_command(CommandType::ClockIn, hash, &conn)?;
        //get number clock-out commands
        let coc  = self.count_command(CommandType::ClockOut, hash, &conn)?;

        info!("clock-in count: {}",cic);
        info!("clock-out count: {}",coc);

        Ok(cic > coc)
    }

    ///Count the number of times a command (clock-in or clock-out) has been inserted into db
    pub fn count_command(&self, cmd_type: CommandType, hash: i64, conn:&Connection)->Result<i16, Report> {
       let mut count_stm = conn.prepare("select count(*) from clock_rust_tasks where command = ?1 and hash = ?2 ")?;
        let mut rows = count_stm.query([cmd_type.to_string(), hash.to_string()])?;
        if let Some(i) = rows.next()?{
            Ok(i.get(0)?)
        }else{
            Ok(0)
        }
    }

    ///Create string with breakdown of tasks, start and end
    /// Optionally limited by time
    /// Optionally limited to a specific task
    pub fn time_report(&self, opt_start:Option<Utc>, opt_end:Option<Utc>, opt_task:Option<&str>)->Result<Vec<Command>, Report>{
        let conn = Connection::open(&self.connection_string)?;
        let mut sql = "select command, task, cmd_date from clock_rust ".to_string();
        let mut args = Vec::new();
        let mut where_inserted = false;

        //okay let's try this with straight up strings
        if let Some(start) = opt_start{
            if !where_inserted  {
                sql += " WHERE ";
                where_inserted = true;
            }

            sql += " cmd_date >= ?";
            args.push(start.to_string());
        };

        if let Some(end) = opt_start{
            if !where_inserted  {
                sql += " WHERE ";
                where_inserted = true;
            }else{
                sql += " AND ";
            }

            sql += " cmd_date <= ?";
            args.push(end.to_string());
        };

        sql += " ORDER BY cmd_date DESC";
        if let Some(task) = opt_task{
            if !where_inserted  {
                sql += " WHERE ";
                where_inserted = true;
            }else{
                sql += " AND ";
            }

            sql += " hash = ? ";

            let mut hasher = DefaultHasher::new();
            task.hash(&mut hasher);
            let hash = hasher.finish() as i64;
            args.push(hash.to_string());
        };

        let mut stmt = conn.prepare(&sql)?;
        let cmds_result = stmt
            .query_map(rusqlite::params_from_iter(args.iter()), |row| {
                let cs:String = row.get_unwrap(0);
                // let command = CommandType::from_str(&cs)?;
                let command = match cs.parse(){
                    Ok(cmd_type) => cmd_type,
                    Err(e) => return Result::Err(e),
                };
                let task = row.get_unwrap(1);
                let cmd_datetime:DateTime<Utc> = row.get_unwrap(2);
               Ok(Command{
                   command,
                   task,
                   cmd_datetime,
               })
            });

        let mut cmds = Vec::new();
        for res in cmds_result {
           cmds.push(res.unwrap()) ;
        }

        cmds

    }

}

#[cfg(test)]
mod tests{
    use chrono::Utc;
    use crate::config;

    use crate::command::CommandType;
    use super::*;

    const TEST_DB_STRING: &str = "./clock_rust_test";
    const TEST_TASK: &str = "Test test data";

    #[test]
    fn test_create_table()->Result<(), Report>{
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
            let mut stmt = conn.prepare("SELECT 1 FROM sqlite_master WHERE type='table' AND name='clock_rust_tasks'")?;
            let mut rows = stmt.query([])?;
            let table_count = if let Some(row) = rows.next()?{
                 row.get_unwrap(0)
            }else{ 0 };
            //delete the file
            std::fs::remove_file(TEST_DB_STRING).expect("could not delete test sqlite db file");
            assert_eq!(table_count, 1)

        }else{
            panic!("Failed to get connection");
        }

        Ok(())
    }

    #[test]
    fn test_run_clock_in_command()->Result<(), Report>{
        config::setup_test_logging();
        let cr = ClockRuster::init(TEST_DB_STRING);
        if let Ok(conn) = Connection::open(cr.connection_string.clone()){
            cr.ensure_storage_exists(&conn)?;
            let cmd = Command::new(CommandType::ClockIn, Utc::now(), "Test test data".to_string());
            match cr.run_clock_command(&cmd) {
                Ok(_)=>println!("Successfully ran clock in command: {} ", cmd),
                Err(why)=>panic!("Unable to run command: {}", why),
            }
        }
        std::fs::remove_file(TEST_DB_STRING).expect("could not delete test sqlite db file");
        Ok(())
    }

    #[test]
    fn test_curently_tracking()->Result<(), Report>{
        config::setup_test_logging();

        let cr = ClockRuster::init(TEST_DB_STRING);
        if let Ok(conn) = Connection::open(cr.connection_string.clone()){
            cr.ensure_storage_exists(&conn)?;
            let cmd = Command::new(CommandType::ClockIn, Utc::now(), TEST_TASK.to_string());
            //we clock in, we should now be tracking
            match cr.run_clock_command(&cmd) {
                Ok(_)=>{
                    match cr.currently_tracking(TEST_TASK){
                        Ok(tracking) => assert!(tracking),
                        Err(why) => panic!("Unable to perform tracking query: {}", why),
                    }
                },
                Err(why)=>panic!("Unable to run command: {}", why),
            };

            //now we clockout - should no longer be tracking
            let cmd = Command::new(CommandType::ClockOut, Utc::now(), TEST_TASK.to_string());
            match cr.run_clock_command(&cmd){
                Ok(_)=>{
                    match cr.currently_tracking(TEST_TASK){
                        Ok(tracking) => assert!(!tracking),
                        Err(why) => panic!("Unable to perform tracking query: {}", why),
                    }
                },
                Err(why)=>panic!("Unable to run command: {}", why),
            }
        }
        std::fs::remove_file(TEST_DB_STRING).expect("could not delete test sqlite db file");
        Ok(())
    }

}

