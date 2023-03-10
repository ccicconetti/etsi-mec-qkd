//! Server that implements an ETSI MEC Lifecycle Management Proxy (LCMP) server.

use crate::applicationlistserver::{build_application_list_server, ApplicationListServer};

/// LCMP server.
pub struct LcmpServer {
    application_list_server: Box<dyn ApplicationListServer + Send + Sync>,
}

impl LcmpServer {
    pub fn application_list(&self) -> &dyn ApplicationListServer {
        self.application_list_server.as_ref()
    }

    pub fn build(als_value: &str) -> Result<LcmpServer, String> {
        Ok(Self {
            application_list_server: build_application_list_server(als_value)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_lcmp() -> Result<(), String> {
        let lcmp = LcmpServer {
            application_list_server: build_application_list_server("empty")?,
        };

        assert!(lcmp.application_list().status().is_ok());

        Ok(())
    }
}
