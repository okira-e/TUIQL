pub mod args;

use crate::app::App;
use crate::cli::args::ConnectCmdArgs;
use crate::cli::args::OpenCmdArgs;
use crate::cli::args::RemoveConnectionCmdArgs;
use crate::cli::args::SaveConnectionCmdArgs;
use crate::config::connection::Connection;
use crate::config::connection::add_connection;
use crate::config::connection::load_connections;
use crate::config::connection::remove_connection;
use crate::config::get_config_dir_path_based_on_os;
use crate::config::project::load_project_config;
use crate::config::settings::load_settings;
use crate::drivers;
use crate::drivers::DbDriver;
use clap::Subcommand;
use color_eyre::Result;
use color_eyre::eyre::bail;
use tabled::Table;
use tabled::Tabled;
use url::Url;

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Connect to a database directly without saving it.
    Connect(ConnectCmdArgs),
    /// Open a saved connection to a database.
    Open(OpenCmdArgs),
    /// List all saved connections.
    #[command(alias = "ls")]
    List,
    ///  Save a new database connection.
    Add(SaveConnectionCmdArgs),
    /// Remove a saved connection by name.
    #[command(alias = "rm")]
    Remove(RemoveConnectionCmdArgs),
}

pub async fn run(args: args::AppArgs) -> Result<()> {
    if args.config_path {
        let path = get_config_dir_path_based_on_os()?;

        match path {
            Some(path) => println!("Config is at {}", path.display()),
            None => {
                let app_name = env!("CARGO_PKG_NAME");
                println!(
                    "Config files are not created yet! Start using {} to create it.",
                    app_name
                )
            }
        };

        return Ok(());
    }

    if let Some(command) = args.command {
        return exec_command(command).await;
    }

    return Ok(());
}

async fn exec_command(command: Commands) -> Result<()> {
    return match command {
        Commands::Connect(args) => connect_directly(args).await,
        Commands::Open(args) => open_connection(args).await,
        Commands::List => list_connections(),
        Commands::Add(args) => save_connection(args).await,
        Commands::Remove(args) => remove_saved_connection(args),
    };
}

async fn run_app(db_driver: DbDriver, project_name: Option<&str>) -> Result<()> {
    let settings = load_settings()?;
    let project_config = match project_name {
        None => None,
        Some(proj) => Some(load_project_config(proj)?),
    };
    let mut app = App::new(settings, db_driver, project_config).await;
    app.init().await?;
    let terminal = ratatui::init();
    let result = app.run(terminal).await;
    ratatui::restore();

    return result;
}

async fn connect_directly(args: args::ConnectCmdArgs) -> Result<()> {
    let db_driver = drivers::new_connection(args.r#type, &args.url).await?;
    return run_app(db_driver, None).await;
}

async fn open_connection(args: args::OpenCmdArgs) -> Result<()> {
    let connections = load_connections()?;

    let connection = match connections.iter().find(|c| c.name == args.connection_name) {
        Some(c) => c,
        None => {
            bail!(
                "The connection you provided doesn't exist. See \"{} ls\" for a list of connections.",
                env!("CARGO_PKG_NAME")
            );
        }
    };

    let db_driver = drivers::new_connection(connection.kind, &connection.url).await?;
    return run_app(db_driver, Some(&connection.name)).await;
}

async fn save_connection(args: args::SaveConnectionCmdArgs) -> Result<()> {
    let password = rpassword::prompt_password("Database password: ")?;

    drivers::ping_connection(
        args.r#type,
        &args.host,
        args.port,
        &args.user,
        &password,
        &args.database,
    )
    .await?;

    let conn = Connection {
        name: args.name,
        kind: args.r#type,
        url: format!(
            "{}://{}:{}@{}:{}/{}",
            args.r#type, args.user, password, args.host, args.port, args.database
        ),
    };

    add_connection(conn)?;

    list_connections()?;

    return Ok(());
}

fn remove_saved_connection(args: args::RemoveConnectionCmdArgs) -> Result<()> {
    remove_connection(&args.connection_name)?;
    list_connections()?;
    return Ok(());
}

fn list_connections() -> Result<()> {
    let connections = load_connections()?;

    if connections.is_empty() {
        println!("No connections found.");
        return Ok(());
    }

    #[derive(Tabled)]
    struct ExtendedDatabaseConnection {
        name: String,
        scheme: String,
        user: String,
        password: String,
        host: String,
        port: String,
        db_name: String,
    }

    let extended_connections = connections
        .iter()
        .map(|connection| {
            let url = Url::parse(&connection.url).unwrap();

            let scheme = url.scheme().to_string();
            let user = url.username().to_string();
            let password = String::from("****");
            let host = url.host_str().unwrap_or("").to_string();
            let port = url.port_or_known_default().unwrap_or(0).to_string();
            let db_name = url.path().trim_start_matches('/').to_string();

            ExtendedDatabaseConnection {
                name: connection.name.clone(),
                scheme,
                user,
                password,
                host,
                port,
                db_name,
            }
        })
        .collect::<Vec<_>>();

    let mut table = Table::new(extended_connections);
    table.with(tabled::settings::Style::ascii_rounded());

    println!("Saved connections:\n");
    println!("{}", table);

    Ok(())
}
