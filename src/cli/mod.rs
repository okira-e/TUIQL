pub mod commands;
pub mod args;


use color_eyre::{eyre::bail, Result};
use tabled::{Table, Tabled};
use url::Url;

use crate::{app, cli::commands::Commands, db, config};


pub async fn run(args: args::AppArgs) -> Result<()> {
    if args.config_path {
        let path = config::get_config_dir_path_based_on_os()?;

        match path {
            Some(path) => println!("Config is at {}", path.display()),
            None => {
                let app_name = env!("CARGO_PKG_NAME");
                println!("Config files are not created yet. Run \"{} init\" to initialize a config.", app_name)
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
        Commands::Init => {
            if let Err(err) = config::create_config_if_not_exists() {
                bail!("Error creating config files: {}", err);
            }

            println!("Config files are created.");
            Ok(())
        }
        Commands::Connect(args) => {
            connect_directly(args).await
        }
        Commands::Open(args) => {
            open_connection(&args.connection_name).await
        }
        Commands::ListConnections => {
            list_connections()
        }
    };
}

/// Connect to the database directly without saving/opening a project.
async fn connect_directly(args: args::ConnectCmdArgs) -> Result<()> {
    let db_conn = db::new_connection(&args.r#type, &args.url).await?;

    let settings = config::load_settings()?;
    
    let mut app = app::App::new(
        settings,
        db_conn,
    ).await;
    app.init().await?;
    let terminal = ratatui::init();
    let result = app.run(terminal).await;
    
    ratatui::restore();
    
    return result;
}

/// Opens a previously saved database connection.
async fn open_connection(connection_name: &str) -> Result<()> {
    let connections = config::load_connections()?;
    
    let connection = match connections.iter().find(|c| c.name == connection_name) {
        Some(c) => c,
        None => {
            bail!(
                "The connection you provided doesn't exist. See \"{} list-connections\" for a list of connections.",
                env!("CARGO_PKG_NAME")
            );
       }
    };

    let db_conn = db::new_connection(&connection.kind, &connection.url).await?;

    let settings = config::load_settings()?;
    let mut app = app::App::new(
        settings,
        db_conn,
    ).await;
    app.init().await?;
    let terminal = ratatui::init();
    let result = app.run(terminal).await;
   
    ratatui::restore();
    
    return result;
}

fn list_connections() -> Result<()> {
    let connections = config::load_connections()?;
    
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
    
    let extended_connections = connections.iter().map(|connection| {
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
    }).collect::<Vec<_>>();
    
    let mut table = Table::new(extended_connections);
    table.with(tabled::settings::Style::modern());

    println!("Saved connections:\n");
    println!("{}", table);
    
    Ok(())
}

