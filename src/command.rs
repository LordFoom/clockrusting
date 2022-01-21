use std::fmt::{Display, Formatter};
use chrono::{DateTime, FixedOffset, ParseResult, Utc};
use std::hash::{ Hash,Hasher };

use color_eyre::{eyre::eyre, Report, Result};
use tracing::{info};
const COMMAND_EG: &str = "clock-in::2021-10-31T04:10:29.316132167Z::'task description'";

///Available commands
#[derive(Clone)]
pub enum CommandType {
    ClockIn,
    ClockOut,
}

impl Display for CommandType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            CommandType::ClockIn => write!(f, "clock-in"),
            CommandType::ClockOut => write!(f, "clock-out"),
        }
    }
}

impl Hash for CommandType{

    fn hash<H: Hasher>(&self, state: &mut H){
        self.to_string().hash(state);
    }
}

///Struct representing commands to track time
#[derive(Hash)]
pub struct Command {
    pub command: CommandType,
    pub cmd_datetime: DateTime<Utc>,
    pub task:  String,
}


impl Command {
    fn new(cmd: CommandType, cmd_datetime:DateTime<Utc>, task: String) -> Self {
        Self {
            command: cmd,
            cmd_datetime,
            task,
        }
    }

}

impl Display for Command {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.command, self.task)
    }
}


///Create a command from string in following format
/// COMMAND-TYPE::TIME::DESCRIPTION
/// where command-type is 'clock-in' or 'clock-out'
/// TIME is rfc3339 time string
/// DESCRIPTION is the description of the task to be tracked
pub fn create_command(check_str: &str) -> Result<Command, Report> {

    // let task = split.as_str();
    let parts:Vec<&str> = check_str.split("::").collect();
    let cmd = match parts[0]   {
        "clock-in" => CommandType::ClockIn ,
        "clock-out" =>  CommandType::ClockOut ,
        //unsupported command
        _ => return Err(eyre!("Fail, available commands: clock-in | clock-out, eg {}", COMMAND_EG)),
    };

    if parts.len()!=3 {
        return Err(eyre!("FAIL, usage command::time::title, eg {}", COMMAND_EG))
    }
    let time_str = parts[1];
    info!("Here is the  TIME STRING: {} ", time_str);
    let task = parts[2];
    //let's get chronological
    let dtime = match DateTime::parse_from_rfc3339(time_str){
        Ok(dt) => { dt}
        Err(why) => { return Err(eyre!("ParseError: {}\n FAIL: please supply datetime in rfc3339 format, eg: { }", why, COMMAND_EG))}
    };

    Ok(Command::new(cmd, dtime.with_timezone(&Utc), String::from(task)))
    
}

#[cfg(test)]
mod tests {
    use color_eyre::Report;
    use crate::config;

    use super::*;

    ///we try to do the run a command that doesn't exist
    #[test]
    fn test_bad_command() {
        // let cmd_runner = CommandConstructor::new("./test.db".to_string());
        config::setup_test_logging();
        let result = create_command("badcommand");
        let report = result.err().unwrap();
        println!("{}", report);
        assert!(report.to_string().starts_with("Fail, available commands: clock-in | clock-out, eg"));
    }

    #[test]
    fn test_create_clock_in() {
        config::setup_test_logging();
        match create_command("clock-in::2021-12-20T20:22:29.52Z::this is a test"){
            Ok(Command{ command: _, task, cmd_datetime:_}) => { assert_eq!(task.to_string(), "this is a test") }
            Err(why) => {
                println!("We have FAILED: {}", why);
                assert!(false);//let it end
            }
        }
    }

    #[test]
    fn test_create_clock_out(){
        config::setup_test_logging();
        let result = create_command("clock-out::2021-12-20T20:36:23.44Z::this is the clock out test");
        match result{
            Ok(Command{task, command:_, cmd_datetime: _}) => assert_eq!(task.to_string(), "this is the clock out test"),
            Err(why) => {
                println!("We have FAILED: {}", why);
                assert_eq!(false, true);//let it end
            }
        }
    }
}