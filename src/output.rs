use std::fs;
use color_eyre::Report;
use crate::command::Command;

///Write out a neat little file with our time tracking report
pub fn write_tracking_report(file_path: &str, cmd_list:&Vec<Command>) -> Result<(), Report> {
    let mut file_contents = String::new();
    cmd_list.iter()
            .for_each(|cmd| {
                file_contents.push_str(&cmd.to_string());
                file_contents.push('\n');
            });

    fs::write(file_path, file_contents)?;
    Ok(())
}

mod tests {
    use rusqlite::Connection;
    use tracing::info;
    use crate::command::CommandType;
    use crate::config;
    use crate::db::ClockRuster;
    use super::*;

    # [test]
    fn test_write_tracking_report() -> Result<(), Report>{
        config::setup_test_logging();
        let cr = ClockRuster::init(crate::db::tests::TEST_DB_STRING);
        if let Ok(conn) = Connection::open(cr.connection_string()){
            cr.ensure_storage_exists(&conn)?;
            //run a clock-in and clock-out command
            let ci = crate::db::tests::create_test_cmd( CommandType::ClockIn,
                                                        crate::db::tests::TEST_TASK,
                                                        "2022-01-31 17:00:28.974008356+00:00");
            let co = crate::db::tests::create_test_cmd( CommandType::ClockOut,
                                                        crate::db::tests::TEST_TASK,
                                                        "2022-01-31 18:31:28.974008356+00:00");
            cr.run_clock_command(&ci)?;
            cr.run_clock_command(&co)?;
            //let's start by getting all of them
            if let Ok(cmds) = cr.command_list(None, None, None){
                match write_tracking_report("./test_report.txt", &cmds){
                    Ok(()) => info!("Successfully wrote report to ./test_report.txt"),
                    Err(why) => panic!("Could not write test report: {}", why),
                }

            }else{
                panic!("Could not list commands")
            }
        }

        std::fs::remove_file(crate::db::tests::TEST_DB_STRING).expect("could not delete test sqlite db file");
        Ok(())
    }

}