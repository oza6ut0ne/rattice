use std::path::PathBuf;

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
    #[structopt(name = "PORT", default_value = "3000")]
    pub port: u16,

    /// Bind address
    #[structopt(short, long, name = "address", default_value = "::")]
    pub bind_address: String,

    /// Specify document root directory
    #[structopt(short, long, parse(from_os_str))]
    pub docroot: Option<PathBuf>,

    /// Username for Basic Authentication [env: RATTICE_USER]
    #[structopt(short, long)]
    pub username: Option<String>,

    /// Password for Basic Authentication [env: RATTICE_PASS]
    #[structopt(short, long)]
    pub password: Option<String>,

    /// Generate random username and/or password with given length
    #[structopt(short, long, name = "length")]
    random_credencial: Option<u8>,

    /// Specify server certificate file and enable HTTPS
    #[structopt(short, long, parse(from_os_str), requires = "server-key")]
    pub server_cert: Option<PathBuf>,

    /// Specify server key file and enable HTTPS
    #[structopt(short = "k", long, parse(from_os_str), requires = "server-cert")]
    pub server_key: Option<PathBuf>,

    /// Increase log level (-v, -vv, -vvv, -vvvv)
    #[structopt(short, long, parse(from_occurrences))]
    pub verbose: u8,
}

impl Opt {
    pub fn init() -> Opt {
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

        opt.username = opt.username.or_else(|| std::env::var("RATTICE_USER").ok());
        opt.password = opt.password.or_else(|| std::env::var("RATTICE_PASS").ok());

        if opt.username.is_some() && opt.username.clone().unwrap().contains(':') {
            eprintln!("error: Colon ':' is not allowed for username");
            std::process::exit(1);
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

        match opt.verbose.cmp(&3) {
            std::cmp::Ordering::Equal => tracing::trace!("{:?}", opt),
            std::cmp::Ordering::Greater => tracing::trace!("{:#?}", opt),
            _ => {}
        }
        opt
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
