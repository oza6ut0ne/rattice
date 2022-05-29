use std::path::PathBuf;

use anyhow::{bail, Result};
use clap::{AppSettings::DeriveDisplayOrder, Parser};
use rand::Rng;

const RANDOM_CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                                  abcdefghijklmnopqrstuvwxyz\
                                  0123456789\
                                  !\"#$%&'()*+,-./;<=>?@[\\]^_`{|}~";

#[cfg(unix)]
const DEFAULT_BIND_ADDRESS: &str = "::";
#[cfg(windows)]
const DEFAULT_BIND_ADDRESS: &str = "0.0.0.0";

#[derive(Parser, Debug)]
#[clap(name = "Rattice", version, setting(DeriveDisplayOrder))]
#[clap(
    mut_arg("version", |arg| arg.help_heading("FLAGS")),
    mut_arg("help", |arg| arg.help_heading("FLAGS"))
)]
pub struct Opt {
    /// Listen port
    #[clap(name = "PORT", default_value = "3000", env = "RATTICE_PORT")]
    pub port: u16,

    /// Bind address
    #[clap(
        short,
        long,
        name = "ADDRESS",
        default_value = DEFAULT_BIND_ADDRESS,
        env = "RATTICE_BIND_ADDR"
    )]
    pub bind_address: String,

    /// Specify document root directory
    #[clap(short, long, parse(from_os_str), env = "RATTICE_DOCROOT")]
    pub docroot: Option<PathBuf>,

    /// Username for Basic Authentication
    #[clap(short, long, env = "RATTICE_USER", hide_env_values = true)]
    pub username: Option<String>,

    /// Password for Basic Authentication
    #[clap(short, long, env = "RATTICE_PASS", hide_env_values = true)]
    pub password: Option<String>,

    /// Generate random username and/or password with given length
    #[clap(short, long, name = "LENGTH")]
    random_credencial: Option<u8>,

    /// Server certificate for HTTPS
    #[clap(
        short,
        long,
        parse(from_os_str),
        requires = "server-key",
        env = "RATTICE_SERVER_CERT"
    )]
    pub server_cert: Option<PathBuf>,

    /// Server key for HTTPS
    #[clap(
        short = 'k',
        long,
        parse(from_os_str),
        requires = "server-cert",
        env = "RATTICE_SERVER_KEY"
    )]
    pub server_key: Option<PathBuf>,

    /// Prefix for HTML title tag
    #[clap(short, long, default_value = "Rattice", env = "RATTICE_TITLE_PREFIX")]
    pub title_prefix: String,

    /// Disable lazy image loading
    #[clap(help_heading = "FLAGS")]
    #[clap(short, long, env = "RATTICE_EAGER")]
    pub eager: bool,

    /// Use X-Real-IP as client address in logs
    #[clap(help_heading = "FLAGS")]
    #[clap(short = 'x', long, env = "RATTICE_USE_REAL_IP")]
    pub use_real_ip: bool,

    /// Increase log level (-v, -vv, -vvv, -vvvv)
    #[clap(help_heading = "FLAGS")]
    #[clap(short, long, parse(from_occurrences))]
    pub verbose: u8,
}

impl Opt {
    pub fn init() -> Result<Self> {
        let mut opt = Self::parse();
        if std::env::var_os("RUST_LOG").is_none() {
            std::env::set_var(
                "RUST_LOG",
                match opt.verbose {
                    0 => "rattice=info,tower_http=info",
                    1 => "rattice=debug,tower_http=debug",
                    _ => "rattice=trace,tower_http=trace",
                },
            )
        }
        tracing_subscriber::fmt::init();
        match opt.verbose.cmp(&3) {
            std::cmp::Ordering::Equal => tracing::trace!("{:?}", opt),
            std::cmp::Ordering::Greater => tracing::trace!("{:#?}", opt),
            _ => { /* nop. */ }
        }

        if matches!(&opt.username, Some(name) if name.contains(':')) {
            bail!("Colon ':' is not allowed for username");
        }

        if let Some(length) = opt.random_credencial {
            opt.username = opt.username.or_else(|| {
                let username = get_random_string(length);
                tracing::info!("generated random username = {}", username);
                Some(username)
            });
            opt.password = opt.password.or_else(|| {
                let password = get_random_string(length);
                tracing::info!("generated random password = {}", password);
                Some(password)
            });
        }

        if let Some(docroot) = opt.docroot {
            opt.docroot = Some(docroot.canonicalize()?);
        }
        if let Some(cert) = opt.server_cert {
            opt.server_cert = Some(cert.canonicalize()?);
        }
        if let Some(key) = opt.server_key {
            opt.server_key = Some(key.canonicalize()?);
        }

        if !opt.title_prefix.is_empty() && !opt.title_prefix.ends_with(' ') {
            opt.title_prefix.push(' ');
        }

        match opt.verbose.cmp(&3) {
            std::cmp::Ordering::Equal => tracing::trace!("{:?}", opt),
            std::cmp::Ordering::Greater => tracing::trace!("{:#?}", opt),
            _ => { /* nop. */ }
        }
        Ok(opt)
    }
}

fn get_random_string(length: u8) -> String {
    let mut rng = rand::thread_rng();
    (0..length)
        .map(|_| {
            let idx = rng.gen_range(0..RANDOM_CHARSET.len());
            RANDOM_CHARSET[idx] as char
        })
        .collect()
}
