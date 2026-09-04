pub mod args;

use crate::app;
use crate::app::App;
use crate::cli::args::ConnectCmdArgs;
use crate::cli::args::EditConnectionCmdArgs;
use crate::cli::args::OpenCmdArgs;
use crate::cli::args::RemoveConnectionCmdArgs;
use crate::cli::args::RenameProjectCmdArgs;
use crate::cli::args::SaveConnectionCmdArgs;
use crate::config::connection::Connection;
use crate::config::connection::add_connection;
use crate::config::connection::load_connections;
use crate::config::connection::remove_connection;
use crate::config::connection::rename_project;
use crate::config::connection::update_connection;
use crate::config::get_config_dir_path_based_on_os;
use crate::config::project::load_project_config;
use crate::config::settings::load_settings;
use crate::drivers;
use crate::drivers::DbDriver;
use crate::drivers::kinds::DbKind;
use clap::Subcommand;
use color_eyre::Result;
use color_eyre::eyre::bail;
use color_eyre::eyre::eyre;
use tabled::Table;
use tabled::Tabled;
use url::Url;

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Connect to a database directly without saving it.
    Connect(ConnectCmdArgs),
    /// Open a saved connection to a database.
    Open(OpenCmdArgs),
    /// Edit a saved database connection.
    Edit(EditConnectionCmdArgs),
    /// List all saved connections.
    #[command(alias = "ls")]
    List,
    ///  Save a new database connection.
    Add(SaveConnectionCmdArgs),
    /// Remove a saved connection by name.
    #[command(alias = "rm")]
    Remove(RemoveConnectionCmdArgs),
    /// Rename a saved project and its connection.
    Rename(RenameProjectCmdArgs),
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
        Commands::Edit(args) => edit_saved_connection(args).await,
        Commands::List => list_connections(),
        Commands::Add(args) => save_connection(args).await,
        Commands::Remove(args) => remove_saved_connection(args),
        Commands::Rename(args) => rename_saved_project(args),
    };
}

async fn run_app(db_driver: DbDriver, project_name: Option<&str>) -> Result<()> {
    let settings = load_settings()?;
    let project_config = match project_name {
        None => None,
        Some(proj) => Some(load_project_config(proj)?),
    };
    let mut app = App::new(settings, db_driver, project_config).await;
    app::init(&mut app).await?;
    let terminal = ratatui::init();
    let result = app::run(app, terminal).await;
    ratatui::restore();

    return result;
}

async fn connect_directly(args: args::ConnectCmdArgs) -> Result<()> {
    let (url, auth_token) = match args.r#type {
        DbKind::SQLite => {
            let path = args.path.ok_or_else(|| {
                eyre!(
                    "Missing --path for sqlite connection.\n\nUsage for sqlite:\n  {} connect --type sqlite --path <PATH_TO_DB_FILE>",
                    env!("CARGO_PKG_NAME")
                )
            })?;
            (format!("sqlite:{}", path), None)
        }
        DbKind::Turso => {
            let url = args.url.ok_or_else(|| {
                eyre!(
                    "Missing --url for turso connection.\n\nUsage for turso:\n  {} connect --type turso --url libsql://<HOST> --token <TOKEN>",
                    env!("CARGO_PKG_NAME")
                )
            })?;
            let token = args
                .token
                .or_else(|| std::env::var("TURSO_AUTH_TOKEN").ok())
                .ok_or_else(|| {
                    eyre!("Missing auth token for turso connection. Pass --token or set TURSO_AUTH_TOKEN.")
                })?;
            (url, Some(token))
        }
        _ => {
            let url = args.url.ok_or_else(|| {
                eyre!(
                    "Missing --url for {} connection.\n\nUsage for {}:\n  {} connect --type {} --url <CONNECTION_URL>",
                    args.r#type,
                    args.r#type,
                    env!("CARGO_PKG_NAME"),
                    args.r#type
                )
            })?;
            (url, None)
        }
    };

    let db_driver = drivers::new_connection(args.r#type, &url, auth_token.as_deref()).await?;
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

    let db_driver = drivers::new_connection(
        connection.kind,
        &connection.url,
        connection.auth_token.as_deref(),
    )
    .await?;
    return run_app(db_driver, Some(&connection.name)).await;
}

async fn edit_saved_connection(args: args::EditConnectionCmdArgs) -> Result<()> {
    if args.host.is_none()
        && args.user.is_none()
        && args.port.is_none()
        && args.database.is_none()
        && args.path.is_none()
        && args.url.is_none()
        && !args.password
        && !args.token
    {
        bail!("No changes provided");
    }

    let connections = load_connections()?;
    let mut connection = match connections
        .iter()
        .find(|connection| connection.name == args.connection_name)
    {
        Some(connection) => connection.clone(),
        None => bail!("Connection \"{}\" not found", args.connection_name),
    };

    match connection.kind {
        DbKind::SQLite => {
            let unsupported = [
                ("--host", args.host.is_some()),
                ("--user", args.user.is_some()),
                ("--port", args.port.is_some()),
                ("--database", args.database.is_some()),
                ("--url", args.url.is_some()),
                ("--password", args.password),
                ("--token", args.token),
            ];
            reject_unsupported_options(connection.kind, &unsupported)?;

            if let Some(path) = args.path {
                drivers::ping_sqlite_connection(&path).await?;
                connection.url = format!("sqlite:{}", path);
            }
        }
        DbKind::Turso => {
            let unsupported = [
                ("--host", args.host.is_some()),
                ("--user", args.user.is_some()),
                ("--port", args.port.is_some()),
                ("--database", args.database.is_some()),
                ("--path", args.path.is_some()),
                ("--password", args.password),
            ];
            reject_unsupported_options(connection.kind, &unsupported)?;

            if let Some(url) = args.url {
                connection.url = url;
            }
            if args.token {
                connection.auth_token = Some(rpassword::prompt_password("Turso auth token: ")?);
            }

            let token = connection
                .auth_token
                .as_deref()
                .ok_or_else(|| eyre!("This Turso connection has no auth token. Use --token to set one."))?;
            drivers::ping_turso_connection(&connection.url, token).await?;
        }
        DbKind::Postgres | DbKind::MySQL | DbKind::Mariadb => {
            let unsupported = [
                ("--path", args.path.is_some()),
                ("--url", args.url.is_some()),
                ("--token", args.token),
            ];
            reject_unsupported_options(connection.kind, &unsupported)?;

            let mut url = Url::parse(&connection.url)?;

            if let Some(host) = args.host {
                url.set_host(Some(&host)).map_err(|_| eyre!("Invalid host: {}", host))?;
            }
            if let Some(user) = args.user {
                url.set_username(&user).map_err(|_| eyre!("Invalid user: {}", user))?;
            }
            if let Some(port) = args.port {
                url.set_port(Some(port)).map_err(|_| eyre!("Invalid port: {}", port))?;
            }
            if let Some(database) = args.database {
                url.set_path(&format!("/{}", database.trim_start_matches('/')));
            }
            if args.password {
                let password = rpassword::prompt_password("Database password: ")?;
                url.set_password(Some(&password))
                    .map_err(|_| eyre!("Unable to set the database password"))?;
            }

            connection.url = url.to_string();
            let _ = drivers::new_connection(connection.kind, &connection.url, None).await?;
        }
    }

    update_connection(&args.connection_name, connection)?;

    println!(
        "Successfully updated connection \"{}\".",
        args.connection_name
    );

    return Ok(());
}

fn reject_unsupported_options(kind: DbKind, options: &[(&str, bool)]) -> Result<()> {
    let unsupported = options
        .iter()
        .filter_map(|(name, provided)| provided.then_some(*name))
        .collect::<Vec<_>>();

    if !unsupported.is_empty() {
        bail!(
            "The following options are not supported for {} connections: {}",
            kind,
            unsupported.join(", ")
        );
    }

    return Ok(());
}

async fn save_connection(args: args::SaveConnectionCmdArgs) -> Result<()> {
    let conn = match args.r#type {
        DbKind::SQLite => {
            let path = args.path.ok_or_else(|| {
                eyre!(
                    "Missing --path for sqlite connection.\n\nUsage for sqlite:\n  {} add --type sqlite --name <NAME> --path <PATH_TO_DB_FILE>",
                    env!("CARGO_PKG_NAME")
                )
            })?;

            drivers::ping_sqlite_connection(&path).await?;

            Connection {
                name: args.name,
                kind: args.r#type,
                url: format!("sqlite:{}", path),
                auth_token: None,
            }
        }
        DbKind::Turso => {
            let url = args.url.ok_or_else(|| {
                eyre!(
                    "Missing --url for turso connection.\n\nUsage for turso:\n  {} add --type turso --name <NAME> --url libsql://<HOST>",
                    env!("CARGO_PKG_NAME")
                )
            })?;

            let token = rpassword::prompt_password("Turso auth token: ")?;

            drivers::ping_turso_connection(&url, &token).await?;

            Connection {
                name: args.name,
                kind: args.r#type,
                url,
                auth_token: Some(token),
            }
        }
        _ => {
            let missing: Vec<&str> = [
                args.host.is_none().then_some("--host"),
                args.user.is_none().then_some("--user"),
                args.port.is_none().then_some("--port"),
                args.database.is_none().then_some("--database"),
            ]
            .into_iter()
            .flatten()
            .collect();

            if !missing.is_empty() {
                bail!(
                    "Missing required options for {} connection: {}\n\nUsage for {}:\n  {} add --type {} --name <NAME> --host <HOST> --port <PORT> --user <USER> --database <DATABASE>",
                    args.r#type,
                    missing.join(", "),
                    args.r#type,
                    env!("CARGO_PKG_NAME"),
                    args.r#type
                );
            }

            let host = args.host.unwrap();
            let user = args.user.unwrap();
            let port = args.port.unwrap();
            let database = args.database.unwrap();

            let password = rpassword::prompt_password("Database password: ")?;

            drivers::ping_connection(args.r#type, &host, port, &user, &password, &database).await?;

            Connection {
                name: args.name,
                kind: args.r#type,
                url: format!(
                    "{}://{}:{}@{}:{}/{}",
                    args.r#type, user, password, host, port, database
                ),
                auth_token: None,
            }
        }
    };

    add_connection(conn)?;

    println!("Saved connections:\n");
    list_connections()?;

    return Ok(());
}

fn remove_saved_connection(args: args::RemoveConnectionCmdArgs) -> Result<()> {
    remove_connection(&args.connection_name)?;
    println!("Successfully removed the connection.");

    return Ok(());
}

fn rename_saved_project(args: args::RenameProjectCmdArgs) -> Result<()> {
    rename_project(&args.current_name, &args.new_name)?;
    println!(
        "Successfully renamed project \"{}\" to \"{}\".",
        args.current_name, args.new_name
    );

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
            if connection.url.starts_with("sqlite:") {
                let path = connection.url.trim_start_matches("sqlite:");
                return ExtendedDatabaseConnection {
                    name: connection.name.clone(),
                    scheme: "sqlite".to_string(),
                    user: String::new(),
                    password: String::new(),
                    host: path.to_string(),
                    port: String::new(),
                    db_name: String::new(),
                };
            }

            if matches!(connection.kind, DbKind::Turso) {
                let host = Url::parse(&connection.url)
                    .ok()
                    .and_then(|u| u.host_str().map(|s| s.to_string()))
                    .unwrap_or_else(|| connection.url.clone());
                return ExtendedDatabaseConnection {
                    name: connection.name.clone(),
                    scheme: "turso".to_string(),
                    user: String::new(),
                    password: if connection.auth_token.is_some() {
                        "****".to_string()
                    } else {
                        String::new()
                    },
                    host,
                    port: String::new(),
                    db_name: String::new(),
                };
            }

            let url = Url::parse(&connection.url).unwrap();

            let scheme = url.scheme().to_string();
            let user = url.username().to_string();
            let password = String::from("****");
            let host = url.host_str().unwrap_or("").to_string();
            let port = url.port_or_known_default().unwrap_or(0).to_string();
            let db_name = url.path().trim_start_matches('/').to_string();

            return ExtendedDatabaseConnection {
                name: connection.name.clone(),
                scheme,
                user,
                password,
                host,
                port,
                db_name,
            };
        })
        .collect::<Vec<_>>();

    let mut table = Table::new(extended_connections);
    table.with(tabled::settings::Style::rounded());

    println!("{}", table);

    Ok(())
}
