//! Server that keeps the internal state of the system components.

use crate::messages::{application_list_from_file, ApplicationList};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fs::File;
use std::io::Write;

/// Return the current ApplicationList to be returned the device apps.
pub trait ApplicationListServer {
    fn application_list(&self) -> Result<ApplicationList, String>;
}

/// Static ApplicationList store.
struct StaticApplicationListServer {
    app_list: Option<ApplicationList>,
    last_err: Option<String>,
}

impl StaticApplicationListServer {
    fn from_file(filename: &str) -> Self {
        let res = File::open(filename);
        match res {
            Ok(mut x) => match application_list_from_file(&mut x) {
                Ok(a) => Self {
                    app_list: Some(a),
                    last_err: None,
                },
                Err(err) => Self {
                    app_list: None,
                    last_err: Some(err.to_string()),
                },
            },
            Err(err) => Self {
                app_list: None,
                last_err: Some(err.to_string()),
            },
        }
    }

    fn empty() -> Self {
        Self {
            app_list: None,
            last_err: None,
        }
    }
}

impl ApplicationListServer for StaticApplicationListServer {
    fn application_list(&self) -> Result<ApplicationList, String> {
        match &self.last_err {
            Some(err) => Err(err.clone()),
            None => match &self.app_list {
                Some(x) => Ok(x.clone()),
                None => Ok(ApplicationList::empty()),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const APP_LIST_JSON_FILE: &str = "application_list.json";

    fn write_example_application_list_to_file() -> Result<(), std::io::Error> {
        let mut f = File::create(APP_LIST_JSON_FILE)?;
        f.write(
            r#"
        {
            "appList": [
                {
                    "appInfo": {
                        "appDId": "test_appDId",
                        "appName": "test_appName",
                        "appProvider": "test_appProvider",
                        "appSoftVersion": "test_appSoftVersion",
                        "appDVersion": "test_appDVersion",
                        "appDescription": "test_appDescription",
                        "appLocation": []
                    },
                    "vendorSpecificExt": null
                }
            ]
        }"#
            .as_bytes(),
        )?;
        Ok(())
    }

    #[test]
    fn test_static_application_list_server() -> Result<(), String> {
        let s = StaticApplicationListServer::empty();
        let a = s.application_list()?;
        assert!(a.appList.is_empty());

        write_example_application_list_to_file().expect("could not write file");
        let s = StaticApplicationListServer::from_file(APP_LIST_JSON_FILE);
        let a = s.application_list()?;
        assert_eq!(1, a.appList.len());
        println!("{}", a.appList[0]);
        std::fs::remove_file(APP_LIST_JSON_FILE).expect("could not remove file");

        Ok(())
    }
}
