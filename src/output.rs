use std::fs::File;
use color_eyre::Report;
use crate::command::Command;

///Write out a neat  little file with our time tracking report
pub fn write_tracking_report(file_path: &str, cmd_list:&Vec<Command>) -> Result<(), Report> {
    let mut output = File::create(file_path);
    let mut file_contents = String::new();
    cmd_list.iter()
            .for_each(|cmd| {
                file_contents.push_str(&cmd.to_string());
                file_contents.push('\n');
            });

    write!(output, file_contents)?
}