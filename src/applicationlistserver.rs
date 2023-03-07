//! Directory of ETSI MEC applications.

use crate::messages::{application_list_from_file, ApplicationList, ApplicationListInfo};
use std::fs::File;
use std::io::Write;

/// Return the current ApplicationList to be returned the device apps.
pub trait ApplicationListServer {
    fn application_list(&self, info: ApplicationListInfo) -> Result<ApplicationList, String>;
    fn status(&self) -> Result<(), String>;
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
    /// Return an ApplicationList message containing only the matching query.
    fn application_list(&self, info: ApplicationListInfo) -> Result<ApplicationList, String> {
        match &self.last_err {
            Some(err) => Err(err.clone()),
            None => match &self.app_list {
                Some(x) => Ok(ApplicationList {
                    appList: x.matching_info(&info),
                }),
                None => Ok(ApplicationList::empty()),
            },
        }
    }

    /// Return the status based on the last apps configuration.
    fn status(&self) -> Result<(), String> {
        match &self.last_err {
            Some(x) => Err(x.clone()),
            None => Ok(()),
        }
    }
}

/// Factory to build ApplicationListServer objects from a string
pub fn build_application_list_server(
    value: &str,
) -> Result<Box<dyn ApplicationListServer + Send + Sync>, String> {
    if let Some(x) = value.find("static;") {
        if x == 0 {
            let rhs = &value[7..];
            if let Some(x) = rhs.find("file=") {
                if x == 0 {
                    return Ok(Box::new(StaticApplicationListServer::from_file(
                        &value[12..],
                    )));
                }
            }
        }
    } else if value == "empty" {
        return Ok(Box::new(StaticApplicationListServer::empty()));
    }
    Err("could not create the ApplicationListServer".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    const APP_LIST_JSON_FILE: &str = "to_remove.json";

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
    fn test_build_application_list_server() {
        let a = build_application_list_server("non-existing-type");
        assert!(a.is_err());

        let a = build_application_list_server("static;aaa");
        assert!(a.is_err());

        let a = build_application_list_server("static;file");
        assert!(a.is_err());

        let a = build_application_list_server("static;file=non-existing");
        assert!(a.is_ok());
    }

    #[test]
    fn test_static_application_list_server() -> Result<(), String> {
        let s = StaticApplicationListServer::empty();
        let a = s.application_list(ApplicationListInfo::empty())?;
        assert!(a.appList.is_empty());

        write_example_application_list_to_file().expect("could not write file");
        let s = StaticApplicationListServer::from_file(APP_LIST_JSON_FILE);
        let a = s.application_list(ApplicationListInfo::empty())?;
        assert_eq!(1, a.appList.len());
        println!("{}", a.appList[0]);
        std::fs::remove_file(APP_LIST_JSON_FILE).expect("could not remove file");

        Ok(())
    }
}
