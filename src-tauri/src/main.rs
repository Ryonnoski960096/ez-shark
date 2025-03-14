// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// extern crate log;

use anyhow::{anyhow, Result};
use ezshark_lib::cert::{init_ca, CertificateAuthority};
use log::debug;
use simplelog::{
    format_description, ColorChoice, CombinedLogger, ConfigBuilder, LevelFilter, TermLogger,
    TerminalMode, WriteLogger,
};
use std::{
    fs,
    path::{Path, PathBuf},
};

const CA_CERT_FILENAME: &str = "ezshark-ca-cert.cer";
const PRIVATE_KEY_FILENAME: &str = "ezshark-key.pem";

// todo 去除全局的unwrap 改为在界面报错
#[tokio::main]
async fn main() -> Result<()> {
    // 初始化
    let config_dir = ensure_config_dir(".ez_shark")?;
    // setup_logger(&config_dir)?;
    debug!("config dir: {:?}", config_dir);

    tokio_rustls::rustls::crypto::ring::default_provider()
        .install_default()
        .expect("Failed to install rustls crypto provider");

    let ca = setup_ca(&config_dir)?;

    // let ip = IpAddr::from([127, 0, 0, 1]);
    // let port = 8081;
    // let listener = TcpListener::bind(SocketAddr::new(ip, port)).await?;

    // let server = ServerBuilder::new(Arc::new(ca))
    // .print_mode(PrintMode::Oneline)
    // .build();

    // let state = server.state();

    // let (ip, port) =
    //     parse_addr(&cli.listen).ok_or_else(|| anyhow!("Invalid addr '{}'", cli.listen))?;
    // let addr = format!("{}:{}", ip, port);
    // let reverse_proxy_url = cli.reverse_proxy_url.map(sanitize_reverse_proxy_url);
    // let title_filters = parse_title_filters(&cli.filters)?;
    // let mime_filters: Vec<String> = cli.mime_filters.iter().map(|v| v.to_lowercase()).collect();
    // let listener = TcpListener::bind(SocketAddr::new(ip, port)).await?;
    // let is_tui = io::stdout().is_terminal() && (cli.tui || (!cli.dump && !cli.web));
    // let is_dump = cli.dump || (!is_tui && !cli.web);
    // let print_mode = if is_tui {
    //     PrintMode::Nothing
    // } else if is_dump {
    //     PrintMode::Markdown
    // } else {
    //     PrintMode::Oneline
    // };
    // let print_mode = "Oneline";

    // let state = server.state();
    // let stop_server = server.run(listener).await?;
    // info!("HTTP(S) proxy listening at {ip}:{port}");

    ezshark_lib::run(ca, config_dir.join(CA_CERT_FILENAME));
    // if is_tui {
    //     let addr = addr.clone();
    //     tui::run(state, &addr).await.context("Failed to run TUI")?;
    // } else {
    //     eprintln!("HTTP(S) proxy listening at {addr}");
    //     if cli.web {
    //         eprintln!(
    //             "Web interface accessible at http://{}:{}{}/",
    //             ip, port, WEB_PREFIX
    //         );
    //     }
    //     shutdown_signal().await;
    // }
    // let _ = stop_server.send(());
    // eprintln!("HTTP(S) proxy listening at {ip}:{port}");
    // shutdown_signal().await;

    // let _ = stop_server.send(());

    Ok(())
}

fn setup_ca(config_dir: &Path) -> Result<CertificateAuthority> {
    let ca_cert_file = config_dir.join(CA_CERT_FILENAME);
    let private_key_file = config_dir.join(PRIVATE_KEY_FILENAME);
    let ca = init_ca(&ca_cert_file, &private_key_file)?;
    Ok(ca)
}
fn setup_logger(config_dir: &Path) -> Result<()> {
    let log_level = if cfg!(debug_assertions) {
        LevelFilter::Debug
    } else {
        LevelFilter::Info
    };
    let crate_name = env!("CARGO_CRATE_NAME");
    let config = ConfigBuilder::new()
        .add_filter_allow(crate_name.to_string())
        .set_time_format_custom(format_description!(
            "[year]-[month]-[day]T[hour]:[minute]:[second].[subsecond digits:3]Z"
        ))
        .set_thread_level(LevelFilter::Off)
        .build();
    let log_path = config_dir.join(format!("{crate_name}.log"));
    let log_file = fs::File::create(log_path)?;
    CombinedLogger::init(vec![
        WriteLogger::new(log_level, config.clone(), log_file),
        TermLogger::new(log_level, config, TerminalMode::Mixed, ColorChoice::Auto),
    ])?;
    Ok(())
}

fn ensure_config_dir(dir_name: &str) -> Result<PathBuf> {
    let mut config_dir = dirs::home_dir().ok_or_else(|| anyhow!("No home dir"))?;
    config_dir.push(dir_name);
    if !config_dir.exists() {
        fs::create_dir_all(&config_dir).map_err(|err| {
            anyhow!(
                "Failed to create config dir '{}', {err}",
                config_dir.display()
            )
        })?;
    }
    Ok(config_dir)
}

async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("Failed to install CTRL+C signal handler")
}
