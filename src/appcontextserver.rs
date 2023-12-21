//! AppContext manager of edge applications in an ETSI MEC system.

use crate::messages::{AppContext, UserAppInstanceInfo};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Interface of an AppContextServer.
pub trait AppContextServer {
    /// Create a new application context.
    /// Upon success the passed argument is modified with filled values, as needed.
    fn new_context(&mut self, app_context: &mut AppContext) -> Result<(), String>;
    /// Delete an active context.
    fn del_context(&mut self, context_id: &str) -> Result<(), String>;
    /// Get an active context.
    fn get_context(&mut self, context_id: &str) -> Result<&AppContext, String>;
    /// Update an active context.
    /// Only the callbackReference is allowed to be updated. If the other
    /// fields do not match exactly, then the command is denied.
    fn update_context(&mut self, app_context: &mut AppContext) -> Result<(), String>;
    /// Return all active contexts.
    fn list_contexts(&mut self) -> Result<Vec<String>, String>;
    /// Return the status of the server.
    fn status(&self) -> Result<(), String>;
}

/// Accepts new contexts up to a maximum and always return the same referenceURI.
struct SimpleAppContextServer {
    /// Maximum number of active contexts.
    max_contexts: usize,
    /// Default reference URI to be assigned to all application contexts if a specific one does not match.
    reference_uri_default: Option<String>,
    /// Map of reference URI by appDId
    reference_uri_by_appdid: HashMap<String, String>,
    /// Active application contexts indexed by the context ID.
    app_contexts: HashMap<String, AppContext>,
}

impl SimpleAppContextServer {
    /// Create a SimpleAppContextServer that is empty upon construction and only uses the default reference URI.
    fn default_empty(max_contexts: usize, reference_uri: &str) -> Self {
        Self {
            max_contexts,
            reference_uri_default: Some(reference_uri.to_string()),
            reference_uri_by_appdid: HashMap::new(),
            app_contexts: HashMap::new(),
        }
    }
    /// Create a SimpleAppContextServer that is empty upon construction and uses only reference URIs by AppDId.
    fn appdid_empty(max_contexts: usize, reference_uri_by_appdid: HashMap<String, String>) -> Self {
        Self {
            max_contexts,
            reference_uri_default: None,
            reference_uri_by_appdid,
            app_contexts: HashMap::new(),
        }
    }
}

impl AppContextServer for SimpleAppContextServer {
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

        // Find the reference URI for this request.
        let mut reference_uri = None;
        if let Some(appdid) = &app_context.appInfo.appDId {
            if let Some(uri) = self.reference_uri_by_appdid.get(appdid) {
                reference_uri = Some(uri.clone());
            }
        }

        // If no match found, use the default reference URI, if defined.
        if reference_uri.is_none() && self.reference_uri_default.is_some() {
            reference_uri = Some(self.reference_uri_default.clone().unwrap());
        }

        // Return an error if it was not possible to return a reference URI.
        if reference_uri.is_none() {
            return Err(format!(
                "It was not possible to find a matching reference URI for AppDId: {}",
                app_context
                    .appInfo
                    .appDId
                    .clone()
                    .unwrap_or("unspecified".to_string())
            ));
        }

        // Assign a new random context id.
        app_context.contextId = Some(Uuid::simple(Uuid::new_v4()).to_string());

        // Assign the app instance id and the reference URI.

        app_context
            .appInfo
            .userAppInstanceInfo
            .push(UserAppInstanceInfo::from_reference_uri(
                &reference_uri.unwrap(),
            ));

        // Add to the list of active contexts.
        self.app_contexts
            .insert(app_context.contextId.clone().unwrap(), app_context.clone());

        Ok(())
    }

    /// Delete an active context.
    fn del_context(&mut self, context_id: &str) -> Result<(), String> {
        match self.app_contexts.remove(context_id) {
            Some(_) => Ok(()),
            None => Err(format!("context ID not found: {}", context_id)),
        }
    }

    /// Get an active context.
    fn get_context(&mut self, context_id: &str) -> Result<&AppContext, String> {
        match self.app_contexts.get(context_id) {
            Some(x) => Ok(x),
            None => Err(format!("context ID not found: {}", context_id)),
        }
    }

    /// Update an active context.
    /// Only the callbackReference is allowed to be updated. If the other
    /// fields do not match exactly, then the command is denied.
    fn update_context(&mut self, app_context: &mut AppContext) -> Result<(), String> {
        if let Some(context_id) = &app_context.contextId {
            match self.app_contexts.get_mut(context_id.as_str()) {
                Some(x) => {
                    match x.identical_except_callback_reference(&app_context) {
                        true => {
                            x.callbackReference = app_context.callbackReference.clone();
                            return Ok(());
                        }
                        false => {
                            return Err(format!(
                                "AppContext in the request does not match that in the server"
                            ));
                        }
                    };
                }
                None => return Err(format!("context ID not found: {}", context_id)),
            }
        }
        Err("context ID not specified in the request".to_string())
    }

    /// Return all active contexts.
    fn list_contexts(&mut self) -> Result<Vec<String>, String> {
        Ok(self.app_contexts.iter().map(|x| x.0.to_string()).collect())
    }

    /// Always return good health.
    fn status(&self) -> Result<(), String> {
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ReferenceUriMapByAppDIdElem {
    pub appdid: String,
    pub reference_uri: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SimpleAppContextServerConf {
    max_contexts: usize,
    mapping: Vec<ReferenceUriMapByAppDIdElem>,
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
                        return Ok(Box::new(SimpleAppContextServer::default_empty(
                            x,
                            tokens[1].as_str(),
                        )));
                    }
                }
            }
        }
    } else if let Some(x) = value.find("file;") {
        if x == 0 && value.len() >= 6 {
            let filename = value[5..].to_string();
            let res = std::fs::File::open(&filename);
            match res {
                Ok(mut file) => {
                    let mut content: String = String::new();
                    let _ = std::io::Read::read_to_string(&mut file, &mut content);
                    let res: Result<SimpleAppContextServerConf, serde_json::Error> =
                        serde_json::from_str(content.as_str());
                    if let Ok(conf) = res {
                        let mut reference_uri_by_appdid = HashMap::new();
                        for elem in conf.mapping {
                            reference_uri_by_appdid.insert(elem.appdid, elem.reference_uri);
                        }
                        return Ok(Box::new(SimpleAppContextServer::appdid_empty(
                            conf.max_contexts,
                            reference_uri_by_appdid,
                        )));
                    } else {
                        return Err(format!("invalid input file: {}", &filename));
                    }
                }
                Err(err) => {
                    return Err(format!("could not read from file '{}': {}", &filename, err));
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
    #[ignore]
    fn test_create_example_simple_app_context_server_json() {
        let mut conf = SimpleAppContextServerConf {
            max_contexts: 10,
            mapping: vec![],
        };
        conf.mapping.push(ReferenceUriMapByAppDIdElem {
            appdid: "1".to_string(),
            reference_uri: "uri1".to_string(),
        });
        conf.mapping.push(ReferenceUriMapByAppDIdElem {
            appdid: "2".to_string(),
            reference_uri: "uri2".to_string(),
        });
        let res = std::fs::File::create("reference_uri_mapping.json");
        if let Ok(file) = res {
            assert!(serde_json::to_writer(file, &conf).is_ok());
        }
    }

    #[test]
    fn test_simple_app_context_server() -> Result<(), String> {
        let mut s = SimpleAppContextServer::default_empty(10, "referenceURI");

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
        assert!(s.list_contexts().is_ok());
        assert!(s.list_contexts().unwrap().len() == 10);

        // get the app contexts one by one
        for elem in &all_contexts {
            if let Some(context_id) = elem {
                s.get_context(context_id.as_str())?;
            }
        }

        // fail to get a non-existing app context
        assert!(&s.get_context("not-a-valid-context-id").is_err());

        // adding the 11-th fails
        assert!(&s.new_context(&mut a).is_err());

        // delete one entry
        let a_context_id = all_contexts.iter().next().unwrap().clone().unwrap();
        s.del_context(a_context_id.as_str())?;
        assert!(s.list_contexts().is_ok());
        assert!(s.list_contexts().unwrap().len() == 9);

        // not getting that context fails, too
        assert!(&s.get_context(a_context_id.as_str()).is_err());

        // now it is possible to add a new one
        s.new_context(&mut a)?;

        // update the entry
        let new_callback_reference = "new_callback_reference";
        a.callbackReference = Some(new_callback_reference.to_string());
        s.update_context(&mut a)?;
        if let Some(context_id) = &a.contextId {
            assert!(&s
                .get_context(context_id.as_str())
                .ok()
                .unwrap()
                .identical_except_callback_reference(&a));
            if let Some(x) = &s
                .get_context(context_id.as_str())
                .ok()
                .unwrap()
                .callbackReference
            {
                assert_eq!(x, new_callback_reference,);
            }
        }

        // try to update an entry with a non-matching AppContext
        let mut b =
            AppContext::request_from_name_provider("my_another_app_name", "my_app_provider");
        b.contextId = a.contextId.clone();
        if let Some(context_id) = &a.contextId {
            assert!(!&s
                .get_context(context_id.as_str())
                .ok()
                .unwrap()
                .identical_except_callback_reference(&b));
            assert!(&s.update_context(&mut b).is_err());
        }

        // cannot add another context
        a.contextId = None;
        a.appInfo.userAppInstanceInfo.clear();
        assert!(&s.new_context(&mut a).is_err());

        Ok(())
    }
}
