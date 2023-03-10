//! AppContext manager of edge applications in an ETSI MEC system.

use crate::messages::{AppContext, UserAppInstanceInfo};
use std::collections::HashMap;
use uuid::Uuid;

/// Interface of an AppContextServer.
pub trait AppContextServer {
    /// Create a new application context.
    /// Upon success the passed argument is modified with filled values, as needed.
    fn new_context(&mut self, app_context: &mut AppContext) -> Result<(), String>;
    /// Delete an active context.
    fn del_context(&mut self, context_id: &str) -> Result<(), String>;
    /// Update an active context.
    /// Only the callbackReference is allowed to be updated. If the other
    /// fields do not match exactly, then the command is denied.
    fn update_context(&mut self, app_context: &mut AppContext) -> Result<(), String>;
    /// Return the status of the server.
    fn status(&self) -> Result<(), String>;
}

/// Accepts new contexts up to a maximum and always return the same referenceURI.
struct SingleAppContextServer {
    /// Maximum number of active contexts.
    max_contexts: usize,
    /// Reference URI to be assigned to all application contexts.
    reference_uri: String,
    /// Active application contexts indexed by the context ID.
    app_contexts: HashMap<String, AppContext>,
}

impl SingleAppContextServer {
    /// Create an SingleAppContextServer that is empty upon construction.
    fn empty(max_contexts: usize, reference_uri: &str) -> Self {
        Self {
            max_contexts,
            reference_uri: reference_uri.to_string(),
            app_contexts: HashMap::new(),
        }
    }
}

impl AppContextServer for SingleAppContextServer {
    /// If the maximum number of contexts is exceeded, the command is rejected.
    /// Otherwise the static reference URI is returned upon accepting the next context.
    fn new_context(&mut self, app_context: &mut AppContext) -> Result<(), String> {
        // Maximum number of contexts: error
        if self.app_contexts.len() == self.max_contexts {
            return Err(format!(
                "Maximum number of active contexts reached {}",
                self.max_contexts
            ));
        }

        // Invalid context as a request: error
        if let Err(x) = app_context.valid_request() {
            return Err(x);
        }

        //
        // Accept the incoming request
        //

        // Assign a new random context id.
        app_context.contextId = Some(Uuid::simple(Uuid::new_v4()).to_string());

        // Assign the app instance id and the reference URI.
        app_context
            .appInfo
            .userAppInstanceInfo
            .push(UserAppInstanceInfo::from_reference_uri(&self.reference_uri));

        // Add to the list of active contexts.
        self.app_contexts
            .insert(app_context.contextId.clone().unwrap(), app_context.clone());

        Ok(())
    }

    /// Delete an active context.
    fn del_context(&mut self, context_id: &str) -> Result<(), String> {
        match self.app_contexts.remove(context_id) {
            Some(_) => Ok(()),
            None => Err("context ID not found".to_string()),
        }
    }

    /// Update an active context.
    /// Only the callbackReference is allowed to be updated. If the other
    /// fields do not match exactly, then the command is denied.
    fn update_context(&mut self, app_context: &mut AppContext) -> Result<(), String> {
        if let Some(context_id) = &app_context.contextId {
            match self.app_contexts.get_mut(context_id.as_str()) {
                Some(x) => {
                    x.contextId = app_context.contextId.clone();
                    return Ok(());
                }
                None => return Err(format!("context ID not found: {}", context_id)),
            }
        }
        Err("unspecified context ID".to_string())
    }

    /// Always return good health.
    fn status(&self) -> Result<(), String> {
        Ok(())
    }
}

/// Factory to build ApplicationListServer objects from a string
pub fn build_app_context_server(
    value: &str,
) -> Result<Box<dyn AppContextServer + Send + Sync>, String> {
    if let Some(x) = value.find("single;") {
        if x == 0 {
            let tokens: Vec<String> = value[7..].split(",").map(|x| x.to_string()).collect();
            if tokens.len() == 2 {
                if let Ok(x) = tokens[0].parse::<usize>() {
                    if !tokens[1].is_empty() {
                        return Ok(Box::new(SingleAppContextServer::empty(
                            x,
                            tokens[1].as_str(),
                        )));
                    }
                }
            }
        }
    }
    Err("could not create the AppContextServer".to_string())
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    #[test]
    fn test_build_app_context_server() {
        assert!(build_app_context_server("non-existing-type").is_err());

        assert!(build_app_context_server("single;not-number,URI").is_err());

        assert!(build_app_context_server("single;10").is_err());

        assert!(build_app_context_server("single;10,").is_err());

        assert!(build_app_context_server("single;1,2,3").is_err());
    }

    #[test]
    fn test_single_app_context_server() -> Result<(), String> {
        let mut s = SingleAppContextServer::empty(10, "referenceURI");

        s.status()?;

        // add invalid app context: error
        let mut a = AppContext::request_from_name_provider("my_app_name", "my_app_provider");
        a.contextId = Some("not-empty-context-id".to_string());
        assert!(a.valid_request().is_err());
        assert!(s.new_context(&mut a).is_err());

        // now the app context is valid: add 10
        a.contextId = None;
        assert!(a.valid_request().is_ok());
        let mut all_contexts = HashSet::new();
        let mut all_instances = HashSet::new();
        for _i in 0..10 {
            assert!(s.new_context(&mut a).is_ok());
            all_contexts.insert(a.contextId.clone());
            assert!(a.appInfo.userAppInstanceInfo.len() == 1);
            let info = a.appInfo.userAppInstanceInfo.first().unwrap();
            all_instances.insert(info.appInstanceId.clone());
            assert!(info.referenceURI.as_ref().unwrap() == "referenceURI");
            assert!(info.appLocation.is_none());

            // Reset the AppContext structure so that it can be recycled for another request.
            a.contextId = None;
            a.appInfo.userAppInstanceInfo.clear();
            assert!(a.valid_request().is_ok());
        }
        assert!(all_contexts.len() == 10);
        assert!(all_instances.len() == 10);

        // the 11-th fails
        assert!(&s.new_context(&mut a).is_err());

        // delete one entry
        let a_context_id = all_contexts.iter().next().unwrap().clone().unwrap();
        s.del_context(a_context_id.as_str())?;

        // now it is possible to add a new one
        s.new_context(&mut a)?;

        // update the entry
        a.callbackReference = Some("new_callback_reference".to_string());
        s.update_context(&mut a)?;

        // cannot add another context
        a.contextId = None;
        a.appInfo.userAppInstanceInfo.clear();
        assert!(&s.new_context(&mut a).is_err());

        Ok(())
    }
}
