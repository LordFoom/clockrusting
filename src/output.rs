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