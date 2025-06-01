mod azurite_storage;

#[allow(unused)]
use {
    azurite_storage::AzuriteStorage,
    clap::Parser,
    jlogger_tracing::{JloggerBuilder, LevelFilter, LogTimeFormat, jdebug, jerror, jinfo},
};

#[derive(Parser)]
#[command(author, version, about, long_about=None)]
pub struct Cli {
    /// Azurite url
    #[arg(short, long, default_value_t=String::from("https://127.0.1:10000"))]
    azurite_url: String,

    /// Log file
    #[arg(short, long)]
    log: Option<String>,

    /// create a new container
    #[arg(long)]
    create_container: Option<String>,

    /// delete a container
    #[arg(long)]
    delete_container: Option<String>,

    /// Verbose
    #[arg(short, long, action=clap::ArgAction::Count)]
    verbose: u8,
}

fn main() {
    let cli = Cli::parse();

    let level = match cli.verbose {
        1 => LevelFilter::DEBUG,
        2 => LevelFilter::TRACE,
        _ => LevelFilter::INFO,
    };

    if let Some(log_file) = cli.log.as_deref() {
        JloggerBuilder::new()
            .max_level(level)
            .log_file(Some((log_file, false)))
            .log_console(true)
            .build();
    } else {
        JloggerBuilder::new()
            .max_level(level)
            .log_console(true)
            .build();
    }

    jdebug!(func = "main", line = line!(), note = "start");
    let azurite_storage = AzuriteStorage::new();

    if let Some(container_name) = cli.create_container {
        azurite_storage.create_container(&container_name);
        std::process::exit(0);
    }

    if let Some(container_name) = cli.delete_container {
        azurite_storage.delete_container(&container_name);
        std::process::exit(0);
    }

    let containers = azurite_storage.list_containers();
    if containers.is_empty() {
        jinfo!(note = "No containers found");
    } else {
        for (i, container) in containers.iter().enumerate() {
            jinfo!(No = i + 1, container = container,);
        }
    }
}
