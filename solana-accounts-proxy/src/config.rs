use secrecy::Secret;
use serde::{Deserialize, Serialize};
use std::{
    env, fmt,
    fs::File,
    io::Read,
    net::{IpAddr, Ipv4Addr, SocketAddr},
};

const ERROR_MESSAGE: &str = "Invalid Number of Command-line Arguments. Expected `2` arguments. 
Use `-h` argument for a list of commands";

const HELP_MESSAGE: [&str; 4] = [
    "solana-accounts-proxy",
    "\n",
    "   Example Usage:",
    "       solana-accounts-proxy ../configs/proxy.toml",
];
/// Reads the user configuration input from [stdin] and
/// transforms the input to a [ProxyConfig]
pub fn load_user_config() -> ProxyConfig {
    let mut cli_args = env::args();

    if cli_args.len() > 2 {
        eprintln!("{}", ERROR_MESSAGE);
        std::process::exit(1);
    }

    let cli_input_path = match cli_args.nth(1) {
        Some(path) => match path.as_str() {
            "-h" | "--help" => {
                for value in HELP_MESSAGE {
                    println!("{value:10}");
                }

                std::process::exit(1);
            }
            _ => path,
        },
        None => {
            eprintln!("Invalid commandline args. The path to the `ProxyConfig.toml` file must be passed when running the binary. Try `solana-accounts-proxy -h` for an example"); //TODO Log to facade
            std::process::exit(1);
        }
    };

    match ProxyConfig::load_config(&cli_input_path) {
        Ok(value) => value,
        Err(error) => {
            eprintln!("server error: {}", error); //TODO Log to facade
            std::process::exit(1);
        }
    }
}

/// The configuration of the socket and database
#[derive(Debug, Deserialize)]
pub struct ProxyConfig {
    socket: SocketConfig,
    postgres: PostgresConfig,
}

impl ProxyConfig {
    /// Load the configuration
    pub fn load_config(path: &str) -> anyhow::Result<Self> {
        let mut file = File::open(&path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        let config: ProxyConfig = toml::from_str(&contents)?;

        Ok(config)
    }

    /// Computes the socket address of the IP and port from [ProxyConfig]
    pub fn get_socketaddr(&self) -> SocketAddr {
        SocketAddr::new(IpAddr::V4(self.socket.ip), self.socket.port)
    }

    /// Load postgres configuration
    pub fn postgres_config(&self) -> &PostgresConfig {
        &self.postgres
    }
}

/// Configuration specific to the IP address and port of the proxy server
#[derive(Debug, Serialize, Deserialize)]
pub struct SocketConfig {
    ip: Ipv4Addr,
    port: u16,
}

/// The configuration to pass to the Postgres connection
#[derive(Deserialize)]
pub struct PostgresConfig {
    pub(crate) user: Secret<String>,
    pub(crate) password: Option<Secret<String>>,
    pub(crate) dbname: String,
    pub(crate) host: String,
    pub(crate) port: Option<u16>,
    // Command line options to pass to the Postgres server
    pub(crate) options: Option<String>,
    //  Sets the application name to be reported in statistics and logs
    pub(crate) application_name: Option<String>,
    pub(crate) connect_timeout: Option<u64>,
}

impl PostgresConfig {
    /// Compute the postgres url `postgres://username:password@host/database`
    #[cfg(all(debug_assertions, feature = "dangerous_debug"))]
    pub fn postgres_url(&self) -> String {
        use secrecy::ExposeSecret;

        let password = match &self.password {
            Some(password) => {
                let mut password_formatting = String::from(":");
                password_formatting.push_str(password.expose_secret());

                password_formatting
            }
            None => "".to_owned(),
        };

        let mut url = "postgres://".to_owned();
        url.push_str(self.user.expose_secret());
        if !password.is_empty() {
            url.push_str(&password);
        }
        url.push('@');
        url.push_str(&self.host);
        url.push('/');
        url.push_str(&self.dbname);

        url
    }
}

impl fmt::Debug for PostgresConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PostgresConfig")
            .field("user", &"REDACTED[POSTGRES_USER]")
            .field("password", {
                if self.password.is_some() {
                    &Some(&"REDACTED[POSTGRES_PASSWORD]")
                } else {
                    &Option::<String>::None
                }
            })
            .field("dbname", &self.dbname)
            .field("host", &self.host)
            .field("port", &self.port)
            .field("options", &self.options)
            .field("application_name", &self.application_name)
            .field("connect_timeout", &self.connect_timeout)
            .finish()
    }
}
