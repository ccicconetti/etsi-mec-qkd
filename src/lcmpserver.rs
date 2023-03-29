//! Server that implements an ETSI MEC Lifecycle Management Proxy (LCMP) server.

use crate::appcontextserver::{build_app_context_server, AppContextServer};
use crate::applicationlistserver::{build_application_list_server, ApplicationListServer};

/// LCMP server.
pub struct LcmpServer {
    application_list_server: Box<dyn ApplicationListServer + Send + Sync>,
    app_context_server: Box<dyn AppContextServer + Send + Sync>,
}

impl LcmpServer {
    pub fn application_list(&self) -> &dyn ApplicationListServer {
        self.application_list_server.as_ref()
    }

    pub fn app_context(&self) -> &dyn AppContextServer {
        self.app_context_server.as_ref()
    }

    pub fn build(als_value: &str, acs_value: &str) -> Result<LcmpServer, String> {
        Ok(Self {
            application_list_server: build_application_list_server(als_value)?,
            app_context_server: build_app_context_server(acs_value)?,
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
            app_context_server: build_app_context_server("single;1,URI")?,
        };

        assert!(lcmp.application_list().status().is_ok());
        assert!(lcmp.app_context().status().is_ok());

        Ok(())
    }
}
