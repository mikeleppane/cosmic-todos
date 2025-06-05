use leptos::leptos_dom::logging;
use miette::{Diagnostic, SourceSpan};
use serde::{Deserialize, Serialize};
use std::{
    env,
    fmt::{self, Formatter},
};
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailConfig {
    pub mikko: String,
    pub niina: String,
}

impl EmailConfig {
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.mikko.is_empty() && self.niina.is_empty()
    }

    #[must_use]
    pub fn get(&self, assignee: &TodoAssignee) -> Option<String> {
        let config = get_config().ok()?;
        match assignee {
            TodoAssignee::Mikko => Some(config.emails.mikko.clone()),
            TodoAssignee::Niina => Some(config.emails.niina.clone()),
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = (TodoAssignee, String)> {
        vec![
            (TodoAssignee::Mikko, self.mikko.clone()),
            (TodoAssignee::Niina, self.niina.clone()),
        ]
        .into_iter()
    }
}

impl IntoIterator for &EmailConfig {
    type Item = (TodoAssignee, String);
    type IntoIter = std::vec::IntoIter<(TodoAssignee, String)>;

    fn into_iter(self) -> Self::IntoIter {
        vec![
            (TodoAssignee::Mikko, self.mikko.clone()),
            (TodoAssignee::Niina, self.niina.clone()),
        ]
        .into_iter()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    // Azure Cosmos DB Configuration
    pub cosmos: CosmosConfig,

    // Application Authentication
    pub auth: AuthConfig,

    // Server Configuration
    pub server: ServerConfig,

    // Logging Configuration
    pub logging: LoggingConfig,

    // Email Configuration
    pub emails: EmailConfig, // Uncomment if email config is needed
                             // Add more configuration sections as needed
}

#[cfg(feature = "ssr")]
use axum::extract::FromRef;

use crate::domain::todo::TodoAssignee;
#[cfg(feature = "ssr")]
impl FromRef<()> for AppConfig {
    fn from_ref(_: &()) -> Self {
        get_config()
            .expect("Configuration should be initialized")
            .clone()
    }
}

impl fmt::Display for AppConfig {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "🌌 Cosmic Todos Configuration")?;
        writeln!(f, "═══════════════════════════════")?;
        writeln!(f)?;

        // Azure Cosmos DB Configuration
        writeln!(f, "🗄️  Azure Cosmos DB:")?;
        writeln!(f, "   Database: {}", self.cosmos.database_name)?;
        writeln!(f, "   Container: {}", self.cosmos.container_name)?;
        writeln!(f, "   Max Items: {}", self.cosmos.max_item_count)?;
        writeln!(f, "   Throughput: {} RU/s", self.cosmos.throughput)?;
        writeln!(f, "   URI: {}", self.cosmos.mask_uri())?;
        writeln!(
            f,
            "   Connection: {}",
            if self.cosmos.connection_string.is_empty() {
                "❌ Not Set"
            } else {
                "✅ Configured"
            }
        )?;

        // Authentication Configuration
        writeln!(f, "🔐 Authentication:")?;
        writeln!(f, "   Username: {}", self.auth.username)?;
        writeln!(f, "   Password: {}", self.auth.mask_password())?;
        writeln!(f)?;

        // Server Configuration
        writeln!(f, "🌐 Server:")?;
        writeln!(f, "   Address: {}", self.server_address())?;
        writeln!(f, "   Environment: {}", self.server.environment)?;
        writeln!(f, "   Site Root: {}", self.server.site_root)?;
        writeln!(f)?;

        // Logging Configuration
        writeln!(f, "📝 Logging:")?;
        writeln!(f, "   Level: {}", self.logging.level)?;
        writeln!(f, "   Format: {}", self.logging.format)?;
        writeln!(f)?;

        // Status indicators
        writeln!(f, "📊 Status:")?;
        writeln!(
            f,
            "   Production Mode: {}",
            if self.is_production() {
                "✅ Yes"
            } else {
                "❌ No"
            }
        )?;
        writeln!(
            f,
            "   Development Mode: {}",
            if self.is_development() {
                "✅ Yes"
            } else {
                "❌ No"
            }
        )?;
        writeln!(
            f,
            "   Configuration Valid: {}",
            if self.validate().is_ok() {
                "✅ Yes"
            } else {
                "❌ No"
            }
        )?;

        // Emails
        writeln!(f, "✉️  Emails:")?;
        if self.emails.is_empty() {
            writeln!(f, "   No email configuration found")?;
        } else {
            for (assignee, email) in &self.emails {
                writeln!(f, "   {assignee}: {email}")?;
            }
        }

        writeln!(f, "═══════════════════════════════")?;
        writeln!(f, "🌌 Cosmic Todos is ready to rock!")?;
        writeln!(f, "═══════════════════════════════")?;
        writeln!(f, "🚀 Enjoy your cosmic journey!")?;

        Ok(())
    }
}

impl fmt::Display for Environment {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let (emoji, name) = match self {
            Environment::Development => ("🔧", "Development"),
            Environment::Staging => ("🚀", "Staging"),
            Environment::Production => ("🏭", "Production"),
        };
        write!(f, "{emoji} {name}")
    }
}

impl fmt::Display for LogFormat {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let (emoji, name) = match self {
            LogFormat::Json => ("📋", "JSON"),
            LogFormat::Pretty => ("🎨", "Pretty"),
        };
        write!(f, "{emoji} {name}")
    }
}

impl fmt::Display for CosmosAuthMethod {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let (emoji, name) = match self {
            CosmosAuthMethod::ConnectionString => ("🔑", "Connection String"),
            CosmosAuthMethod::AzureAD => ("🆔", "Azure AD"),
        };
        write!(f, "{emoji} {name}")
    }
}

// Implement safe debug printing for sensitive configurations
impl CosmosConfig {
    fn mask_uri(&self) -> String {
        if self.uri.is_empty() {
            "❌ Not Set".to_string()
        } else {
            // Extract just the account name from the URI for display
            if let Some(start) = self.uri.find("://") {
                let after_protocol = &self.uri[start + 3..];
                if let Some(end) = after_protocol.find('.') {
                    let account_name = &after_protocol[..end];
                    format!("https://{account_name}.documents.azure.com:443/ (✅ Configured)")
                } else {
                    "✅ Configured (URI format)".to_string()
                }
            } else {
                "✅ Configured".to_string()
            }
        }
    }
}

impl AuthConfig {
    fn mask_password(&self) -> String {
        if self.password.is_empty() {
            "❌ Not Set".to_string()
        } else {
            format!("✅ Set ({} characters)", self.password.len())
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CosmosConfig {
    pub uri: String,
    pub connection_string: String,
    pub database_name: String,
    pub container_name: String,
    pub max_item_count: u32,
    pub throughput: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    pub username: String,
    pub password: String,
    pub session_timeout_hours: u64, // in seconds
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub site_root: String,
    pub environment: Environment,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub format: LogFormat,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Environment {
    Development,
    Staging,
    Production,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogFormat {
    Json,
    Pretty,
}

#[derive(Debug, Clone)]
pub enum CosmosAuthMethod {
    ConnectionString,
    AzureAD,
}

#[derive(Diagnostic, Debug, Error)]
pub enum ConfigError {
    #[error("Missing required environment variable")]
    #[diagnostic(
        code(config::missing_env_var),
        help("Set the {name} environment variable"),
        url("https://familyleppanen.net/docs/configuration")
    )]
    MissingRequired {
        name: String,
        #[source_code]
        src: String,
        #[label("this environment variable is missing")]
        span: SourceSpan,
    },

    #[error("Invalid configuration value")]
    #[diagnostic(
        code(config::invalid_value),
        help("Check the configuration documentation for valid values"),
        url("https://familyleppanen.net/docs/configuration")
    )]
    InvalidValue {
        value: String,
        expected: String,
        #[source_code]
        src: String,
        #[label("invalid value found here")]
        span: SourceSpan,
    },

    #[error("Environment variable parse error")]
    #[diagnostic(
        code(config::parse_error),
        help("Ensure the environment variable contains a valid {expected_type}")
    )]
    ParseError {
        var_name: String,
        value: String,
        expected_type: String,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("Azure Cosmos DB configuration error")]
    #[diagnostic(
        code(config::cosmos_config_error),
        help(
            "Either set AZURE_COSMOS_CONNECTION_STRING or both AZURE_COSMOS_ENDPOINT with proper Azure AD authentication"
        ),
        url("https://docs.microsoft.com/en-us/azure/cosmos-db/")
    )]
    CosmosConfigError {
        #[source_code]
        src: String,
        #[label("configuration issue detected here")]
        span: SourceSpan,
    },
}

impl AppConfig {
    /// Load configuration from environment variables with fallback defaults
    ///
    /// # Errors
    ///
    /// Returns a `ConfigError` if required environment variables are missing,
    /// contain invalid values, or if parsing of numeric values fails.
    pub fn from_env() -> Result<Self, ConfigError> {
        {
            // Try to load .env file, but don't fail if it doesn't exist
            if let Err(e) = dotenv::dotenv() {
                // Only log if the error is NOT "file not found"
                if !e.to_string().contains("No such file or directory") {
                    logging::console_log(&format!("Warning: Could not load .env file: {e}"));
                }
            } else {
                logging::console_log("Loaded environment variables from .env file");
            }
        }
        let env_vars = Self::collect_env_vars();

        // Cosmos DB Configuration
        let cosmos = CosmosConfig {
            uri: Self::get_required_env_var("AZURE_COSMOS_DB_URI")?,
            connection_string: Self::get_required_env_var("AZURE_COSMOS_DB_PRIMARY_KEY")?,
            database_name: Self::get_required_env_var("AZURE_COSMOS_DATABASE_NAME")?,
            container_name: Self::get_required_env_var("AZURE_COSMOS_CONTAINER_NAME")?,
            max_item_count: Self::parse_env_var_with_default("AZURE_COSMOS_MAX_ITEM_COUNT", 100)?,
            throughput: Self::parse_env_var_with_default("AZURE_COSMOS_THROUGHPUT", 400)?,
        };

        // Authentication Configuration
        let auth = AuthConfig {
            username: Self::get_required_env_var("COSMIC_USERNAME")?,
            password: Self::get_required_env_var("COSMIC_PASSWORD")?,
            session_timeout_hours: Self::parse_env_var_with_default(
                "COSMIC_SESSION_TIMEOUT_HOURS",
                1,
            )?,
        };

        // Server Configuration
        let server_addr = env_vars
            .get("LEPTOS_SITE_ADDR")
            .cloned()
            .unwrap_or_else(|| "0.0.0.0:3000".to_string());

        let (host, port) = Self::parse_server_address(&server_addr)?;

        let server = ServerConfig {
            host,
            port,
            site_root: env_vars
                .get("LEPTOS_SITE_ROOT")
                .cloned()
                .unwrap_or_else(|| "site".to_string()),
            environment: Self::parse_environment(
                &env_vars
                    .get("ENVIRONMENT")
                    .cloned()
                    .unwrap_or_else(|| "development".to_string()),
            )?,
        };

        // Logging Configuration
        let logging = LoggingConfig {
            level: env_vars
                .get("RUST_LOG")
                .cloned()
                .unwrap_or_else(|| match server.environment {
                    Environment::Production => "info".to_string(),
                    Environment::Staging | Environment::Development => "debug".to_string(),
                }),
            format: Self::parse_log_format(
                &env_vars
                    .get("LOG_FORMAT")
                    .cloned()
                    .unwrap_or_else(|| "pretty".to_string()),
            )?,
        };

        // email is specified in env varialbles as EMAIL_<assignee>=<email>
        let emails = EmailConfig {
            mikko: Self::get_required_env_var("EMAIL_MIKKO")?,
            niina: Self::get_required_env_var("EMAIL_NIINA")?,
        };

        Ok(AppConfig {
            cosmos,
            auth,
            server,
            logging,
            emails,
        })
    }

    fn collect_env_vars() -> std::collections::HashMap<String, String> {
        env::vars().collect()
    }

    fn get_required_env_var(name: &str) -> Result<String, ConfigError> {
        env::var(name).map_err(|_| {
            let config_line = format!("{name}=<missing>");
            ConfigError::MissingRequired {
                name: name.to_string(),
                src: config_line.clone(),
                span: (0, config_line.len()).into(),
            }
        })
    }

    fn parse_env_var_with_default<T>(name: &str, default: T) -> Result<T, ConfigError>
    where
        T: std::str::FromStr,
        T::Err: std::error::Error + Send + Sync + 'static,
    {
        match env::var(name) {
            Ok(value) => value.parse().map_err(|e| ConfigError::ParseError {
                var_name: name.to_string(),
                value: value.clone(),
                expected_type: std::any::type_name::<T>().to_string(),
                source: Box::new(e),
            }),
            Err(_) => Ok(default),
        }
    }

    fn parse_server_address(addr: &str) -> Result<(String, u16), ConfigError> {
        let parts: Vec<&str> = addr.split(':').collect();
        if parts.len() != 2 {
            return Err(ConfigError::InvalidValue {
                value: addr.to_string(),
                expected: "host:port format (e.g., 0.0.0.0:3000)".to_string(),
                src: format!("LEPTOS_SITE_ADDR={addr}"),
                span: (17, addr.len()).into(),
            });
        }

        let host = parts[0].to_string();
        let port = parts[1]
            .parse::<u16>()
            .map_err(|e| ConfigError::ParseError {
                var_name: "LEPTOS_SITE_ADDR".to_string(),
                value: parts[1].to_string(),
                expected_type: "valid port number (1-65535)".to_string(),
                source: Box::new(e),
            })?;

        Ok((host, port))
    }

    fn parse_environment(env_str: &str) -> Result<Environment, ConfigError> {
        match env_str.to_lowercase().as_str() {
            "production" | "prod" => Ok(Environment::Production),
            "staging" | "stage" => Ok(Environment::Staging),
            "development" | "dev" => Ok(Environment::Development),
            _ => Err(ConfigError::InvalidValue {
                value: env_str.to_string(),
                expected: "development, staging, or production".to_string(),
                src: format!("ENVIRONMENT={env_str}"),
                span: (12, env_str.len()).into(),
            }),
        }
    }

    fn parse_log_format(format_str: &str) -> Result<LogFormat, ConfigError> {
        match format_str.to_lowercase().as_str() {
            "json" => Ok(LogFormat::Json),
            "pretty" => Ok(LogFormat::Pretty),
            _ => Err(ConfigError::InvalidValue {
                value: format_str.to_string(),
                expected: "json or pretty".to_string(),
                src: format!("LOG_FORMAT={format_str}"),
                span: (11, format_str.len()).into(),
            }),
        }
    }

    /// Get the full server address
    #[must_use]
    pub fn server_address(&self) -> String {
        format!("{}:{}", self.server.host, self.server.port)
    }

    /// Check if running in production
    #[must_use]
    pub fn is_production(&self) -> bool {
        matches!(self.server.environment, Environment::Production)
    }

    /// Check if running in development
    #[must_use]
    pub fn is_development(&self) -> bool {
        matches!(self.server.environment, Environment::Development)
    }

    /// Validate configuration
    ///
    /// # Errors
    ///
    /// Returns a `ConfigError` if any configuration values are invalid,
    /// such as empty username, password too short, invalid port number,
    /// or insufficient Cosmos DB throughput.
    pub fn validate(&self) -> Result<(), ConfigError> {
        // Validate authentication
        if self.auth.username.is_empty() {
            let config_line = format!("COSMIC_USERNAME={}", self.auth.username);
            return Err(ConfigError::InvalidValue {
                value: self.auth.username.clone(),
                expected: "non-empty username".to_string(),
                src: config_line.clone(),
                span: (15, config_line.len()).into(),
            });
        }

        if self.auth.password.len() < 8 {
            let config_line = format!("COSMIC_PASSWORD={}", "*".repeat(self.auth.password.len()));
            return Err(ConfigError::InvalidValue {
                value: format!("{} characters", self.auth.password.len()),
                expected: "at least 8 characters".to_string(),
                src: config_line.clone(),
                span: (15, config_line.len()).into(),
            });
        }

        // Validate server configuration
        if self.server.port == 0 {
            let config_line = format!("LEPTOS_SITE_ADDR={}:{}", self.server.host, self.server.port);
            return Err(ConfigError::InvalidValue {
                value: self.server.port.to_string(),
                expected: "port number between 1 and 65535".to_string(),
                src: config_line.clone(),
                span: (config_line.len() - 1, 1).into(),
            });
        }

        // Validate Cosmos DB configuration
        if self.cosmos.throughput < 400 {
            let config_line = format!("AZURE_COSMOS_THROUGHPUT={}", self.cosmos.throughput);
            return Err(ConfigError::InvalidValue {
                value: self.cosmos.throughput.to_string(),
                expected: "at least 400 RU/s".to_string(),
                src: config_line.clone(),
                span: (24, config_line.len() - 24).into(),
            });
        }

        Ok(())
    }
}

// Global configuration instance
static APP_CONFIG: std::sync::LazyLock<Result<AppConfig, ConfigError>> =
    std::sync::LazyLock::new(|| {
        let config = AppConfig::from_env()?;
        config.validate()?;
        Ok(config)
    });

/// Get the global application configuration
///
/// Returns a reference to the global application configuration instance.
///
/// # Errors
///
/// Returns a reference to a `ConfigError` if the configuration couldn't be loaded
/// or validation failed.
pub fn get_config() -> Result<&'static AppConfig, &'static ConfigError> {
    APP_CONFIG.as_ref()
}

/// Initialize and validate configuration at startup with pretty error reporting
///
/// # Errors
///
/// Returns a `miette::Error` if the configuration couldn't be loaded or validation failed.
pub fn initialize_config() -> miette::Result<()> {
    match get_config() {
        Ok(config) => {
            {
                logging::console_log("Configuration loaded successfully");
                logging::console_log(&format!("Environment: {:?}", config.server.environment));
                logging::console_log(&format!("Server address: {}", config.server_address()));
            }
            Ok(())
        }
        Err(e) => {
            logging::console_error(&format!("Configuration validation failed: {e}"));
            Err(miette::miette!(e))
        }
    }
}
