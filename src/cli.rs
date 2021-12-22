use std::path::PathBuf;

use anyhow::{bail, Result};
use rand::Rng;
use structopt::{clap::AppSettings::DeriveDisplayOrder, StructOpt};

const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                            abcdefghijklmnopqrstuvwxyz\
                            0123456789\
                            !\"#$%&'()*+,-./;<=>?@[\\]^_`{|}~";

#[derive(StructOpt, Debug)]
#[structopt(name = "Rattice", setting(DeriveDisplayOrder))]
pub struct Opt {
    /// Listen port
    #[structopt(name = "PORT", default_value = "3000", env = "RATTICE_PORT")]
    pub port: u16,

    /// Bind address
    #[structopt(
        short,
        long,
        name = "address",
        default_value = "::",
        env = "RATTICE_BIND_ADDR"
    )]
    pub bind_address: String,

    /// Disable lazy image loading [env: RATTICE_EAGER]
    #[structopt(short, long)]
    eager: bool,

    /// Specify document root directory
    #[structopt(short, long, parse(from_os_str), env = "RATTICE_DOCROOT")]
    pub docroot: Option<PathBuf>,

    /// Username for Basic Authentication
    #[structopt(short, long, env = "RATTICE_USER", hide_env_values = true)]
    pub username: Option<String>,

    /// Password for Basic Authentication
    #[structopt(short, long, env = "RATTICE_PASS", hide_env_values = true)]
    pub password: Option<String>,

    /// Generate random username and/or password with given length
    #[structopt(short, long, name = "length")]
    random_credencial: Option<u8>,

    /// Server certificate file for HTTPS
    #[structopt(
        short,
        long,
        parse(from_os_str),
        requires = "server-key",
        env = "RATTICE_SERVER_CERT"
    )]
    pub server_cert: Option<PathBuf>,

    /// Server key file for HTTPS
    #[structopt(
        short = "k",
        long,
        parse(from_os_str),
        requires = "server-cert",
        env = "RATTICE_SERVER_KEY"
    )]
    pub server_key: Option<PathBuf>,

    /// Increase log level (-v, -vv, -vvv, -vvvv)
    #[structopt(short, long, parse(from_occurrences))]
    pub verbose: u8,
}

impl Opt {
    pub fn init() -> Result<Self> {
        let mut opt = Self::from_args();
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
            _ => {}
        }

        if opt.username.is_some() && opt.username.as_ref().unwrap().contains(':') {
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
            opt.docroot = Some(docroot.canonicalize()?)
        }
        if let Some(cert) = opt.server_cert {
            opt.server_cert = Some(cert.canonicalize()?)
        }
        if let Some(key) = opt.server_key {
            opt.server_key = Some(key.canonicalize()?)
        }

        if opt.eager {
            std::env::set_var("RATTICE_EAGER", "1")
        }

        match opt.verbose.cmp(&3) {
            std::cmp::Ordering::Equal => tracing::trace!("{:?}", opt),
            std::cmp::Ordering::Greater => tracing::trace!("{:#?}", opt),
            _ => {}
        }
        Ok(opt)
    }
}

fn get_random_string(length: u8) -> String {
    let mut rng = rand::thread_rng();
    (0..length)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}
