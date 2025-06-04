#[allow(unused)]
use {
    azuritelib::{azurite_storage::AzuriteStorage, error::AzuriteStorageError},
    clap::Parser,
    error_stack::{Result, ResultExt},
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
    create_container: bool,

    /// delete a container
    #[arg(long)]
    delete_container: bool,

    /// Container name
    #[arg(short, long, default_value_t=String::from("default"))]
    container_name: String,

    /// list blobs
    #[arg(long)]
    list_blobs: bool,

    /// put blob
    #[arg(short, long)]
    put_blob: Option<String>,

    /// Verbose
    #[arg(short, long, action=clap::ArgAction::Count)]
    verbose: u8,
}

fn main() -> Result<(), AzuriteStorageError> {
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
    let azurite_storage = AzuriteStorage::new(&cli.azurite_url)?;

    if cli.create_container {
        azurite_storage.create_container(&cli.container_name)?;
        std::process::exit(0);
    }

    if cli.delete_container {
        azurite_storage.delete_container(&cli.container_name)?;
        std::process::exit(0);
    }

    if cli.list_blobs {
        let blobs = azurite_storage.list_blobs(&cli.container_name)?;
        if blobs.is_empty() {
            jinfo!(
                note = "No blobs found in container",
                container = cli.container_name
            );
        } else {
            for (i, blob) in blobs.iter().enumerate() {
                jinfo!(
                    No = i + 1,
                    blob = blob.name,
                    version = blob.version_id.as_deref().unwrap_or("N/A"),
                    content_type = blob.properties.content_type,
                    etag = blob.properties.etag.to_string(),
                );
            }
        }
        std::process::exit(0);
    }

    if let Some(file_path) = cli.put_blob {
        azurite_storage.push_blob(&cli.container_name, &file_path)?;
        jinfo!(
            note = "File uploaded successfully",
            file = file_path,
            container = cli.container_name
        );
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

    Ok(())
}
