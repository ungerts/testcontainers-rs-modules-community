use testcontainers::{
    core::{ContainerPort, WaitFor},
    Image,
};
use std::collections::BTreeMap;
use std::borrow::Cow;

const NAME: &str = "valkey/valkey";
const TAG: &str = "8.0.2-alpine";

/// Default port (6379) on which Valkey is exposed
pub const VALKEY_PORT: ContainerPort = ContainerPort::Tcp(6379);

/// Module to work with [`Valkey`] inside of tests.
/// Valkey is a high-performance data structure server that primarily serves key/value workloads.
///
/// Starts an instance of Valkey based on the official [`Valkey docker image`].
///
/// By default, Valkey is exposed on Port 6379 ([`VALKEY_PORT`]), just like Redis, and has no access control.
/// Currently, for communication with Valkey we can still use redis library.
///
/// # Example
/// ```
/// use redis::Commands;
/// use testcontainers_modules::{
///     testcontainers::runners::SyncRunner,
///     valkey::{Valkey, VALKEY_PORT},
/// };
///
/// let valkey_instance = Valkey::default().start().unwrap();
/// let host_ip = valkey_instance.get_host().unwrap();
/// let host_port = valkey_instance.get_host_port_ipv4(VALKEY_PORT).unwrap();
///
/// let url = format!("redis://{host_ip}:{host_port}");
/// let client = redis::Client::open(url.as_ref()).unwrap();
/// let mut con = client.get_connection().unwrap();
///
/// con.set::<_, _, ()>("my_key", 42).unwrap();
/// let result: i64 = con.get("my_key").unwrap();
/// ```
///
/// [`Valkey`]: https://valkey.io/
/// [`Valeky docker image`]: https://hub.docker.com/r/valkey/valkey
/// [`VALKEY_PORT`]: super::VALKEY_PORT
#[derive(Debug, Default, Clone)]
pub struct Valkey {
    /// (remove if there is another variable)
    /// Field is included to prevent this struct to be a unit struct.
    /// This allows extending functionality (and thus further variables) without breaking changes
    //_priv: (),
    env_vars: BTreeMap<String, String>,
    tag: Option<String>,
}

impl Valkey {
     /// Create a new AnvilNode with the latest Foundry image
     pub fn latest() -> Self {
        Self {
            tag: Some("latest".to_string()),
            ..Default::default()
        }
    }

    /// Add extra flags
    pub fn with_valkey_extra_flags(self, valkey_extra_flags: String) -> Self {
        let mut env_vars = self.env_vars;
        env_vars.insert("VALKEY_EXTRA_FLAGS".to_string(), valkey_extra_flags);
        Self { env_vars, tag: self.tag }
    }
}

impl Image for Valkey {
    fn name(&self) -> &str {
        NAME
    }

    fn tag(&self) -> &str {
        self.tag.as_deref().unwrap_or(TAG)
    }

    fn ready_conditions(&self) -> Vec<WaitFor> {
        vec![WaitFor::message_on_stdout("Ready to accept connections")]
    }

    fn env_vars(
        &self,
    ) -> impl IntoIterator<Item = (impl Into<Cow<'_, str>>, impl Into<Cow<'_, str>>)> {
        &self.env_vars
    }

}

#[cfg(test)]
mod tests {
    use redis::Commands;

    use crate::{testcontainers::runners::SyncRunner, valkey::Valkey};

    #[test]
    fn valkey_fetch_an_integer() -> Result<(), Box<dyn std::error::Error + 'static>> {
        let _ = pretty_env_logger::try_init();
        let node = Valkey::default().start()?;
        let host_ip = node.get_host()?;
        let host_port = node.get_host_port_ipv4(6379)?;
        let url = format!("redis://{host_ip}:{host_port}");

        let client = redis::Client::open(url.as_ref()).unwrap();
        let mut con = client.get_connection().unwrap();
        
        con.set::<_, _, ()>("my_key", 42).unwrap();
        let result: i64 = con.get("my_key").unwrap();
        assert_eq!(42, result);
        Ok(())
    }

    #[test]
    fn valkey_latest() -> Result<(), Box<dyn std::error::Error + 'static>> {
        let _ = pretty_env_logger::try_init();
        let node = Valkey::latest().start()?;
        let tag = node.image().tag.clone();
        assert_eq!(Some("latest".to_string()), tag);
        Ok(())
    }
}
