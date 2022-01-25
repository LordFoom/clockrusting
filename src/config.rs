use std::sync::Once;
use tracing_subscriber;
use tracing::{Level};
use tracing_subscriber::FmtSubscriber;

// pub fn setup(verbose: bool, log_file_option:Option<String>) ->Result<(), Report>{
//     if std::env::var("RUST_LIB_BACKTRACE").is_err(){
//         std::env::set_var("RUST_LIB_BACKTRACE", "1")
//     }
//     //if no name, default
//     let log_file = if let Some(name) = log_file_option{
//         name
//     }else{
//         String::from("clockrust.log")
//     };
//     color_eyre::install()?;
//
//     if verbose {
//         std::env::set_var("RUST_LOG","info")
//     }
//
//     let file_appender = RollingFileAppender::new(Rotation::NEVER, ".", log_file);
//     let (nb_file_appender, _guard) = tracing_appender::non_blocking(file_appender);
//
//     tracing_subscriber::fmt()
//         .with_writer(nb_file_appender)
//         .init();
//
//     info!("Logging initialized successfully.");
//     // tracing_subscriber::fmt::fmt()
//     //     .with_env_filter(EnvFilter::from_default_env())
//     //     .init();
//     Ok(())
// }

static INIT: Once = Once::new();

pub fn setup_test_logging(){
    INIT.call_once(||{
        let subs = FmtSubscriber::builder()
            .with_max_level(Level::TRACE)
            .finish();

        tracing::subscriber::set_global_default(subs).expect("setting stdout logger failed");
    });
}
//Get our cli arguments and return them in a nice data structure
// Current args:
// verbose: log stuff
// port: listen here
// file: sqlite db file
// pub fn parse_args()->ArgMatches{
//     App::new("clockrust")
//         .version("0.1")
//         .author("foom")
//         .about("Time tracking server and app")
//         .arg("-v, --verbose 'Log much information' ")
//         .arg("-p, --port 'Port number'")
//         .arg("-f, --file 'SQLite file where we store times'")
//         .get_matches()
// }
