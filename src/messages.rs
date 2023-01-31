//! Messages according to the following specifications:
//! ETSI GS MEC 016 V2.2.1 (2020-04)

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashSet;
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

/// Validate a message (or element thereof).
pub trait Validate {
    fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

/// Return error if the vector of problems passed is not empty.
fn check(problems: Vec<String>) -> Result<(), String> {
    if problems.is_empty() {
        Ok(())
    } else {
        Err(problems.join(";").to_string())
    }
}

/// Add a problem to the list if validation fails.
fn add_problem<T>(item: &T, problems: &mut Vec<String>)
where
    T: Validate,
{
    match item.validate() {
        Ok(()) => (),
        Err(err) => problems.push(err),
    }
}

/// ProblemDetails data type, as specified in IETF RFC 7807 and specialized
/// in ETSI GS MEC 009 V2.2.1 (2020-10) Table 6.15.3-1.
#[derive(Serialize, Deserialize, Clone)]
pub struct ProblemDetails {
    /// The HTTP status code for this occurrence of the problem.
    pub status: usize,
    /// A human-readable explanation specific to this occurrence of the problem.
    pub detail: String,
}

/// Polygon as defined in RFC 7946.
/// The first element in the array represents the exterior ring.
/// Any subsequent elements represent interior rings (or holes).
///
/// Examples with holes:
///
///    With holes:
///
/// {
///     "type": "Polygon",
///     "coordinates": [
///         [
///             [100.0, 0.0],
///             [101.0, 0.0],
///             [101.0, 1.0],
///             [100.0, 1.0],
///             [100.0, 0.0]
///         ],
///         [
///             [100.8, 0.8],
///             [100.8, 0.2],
///             [100.2, 0.2],
///             [100.2, 0.8],
///             [100.8, 0.8]
///         ]
///     ]
/// }
#[derive(Serialize, Deserialize, Clone)]
pub struct Polygon {
    coordinates: Vec<Vec<Vec<f64>>>,
}

/// civicAddressElement in a LocationConstraints informantion element
#[derive(Serialize, Deserialize, Clone)]
pub struct CivicAddressElement {
    /// Describe the content type of caValue.
    /// The value of caType shall comply with section 3.4 of IETF RFC 4776.
    caType: i32,
    /// Content of civic address element corresponding to the caType.
    /// The format caValue shall comply with section 3.4 of IETF RFC 4776.
    caValue: String,
}

/// LocationConstraints information element
#[derive(Serialize, Deserialize, Clone)]
pub struct LocationConstraints {
    /// The two-letter ISO 3166 [7] country code in capital letters.
    /// Shall be present in case the "area" attribute is absent.
    /// May be absent if the "area" attribute is present.
    countryCode: Option<String>,
    /// Zero or more elements comprising the civic address.
    /// Shall be absent if the "area" attribute is present.
    civicAddressElement: Vec<CivicAddressElement>,
    /// Geographic area.
    /// Shall be absent if the "civicAddressElement" attribute is present.
    /// The content of this attribute shall follow the provisions for the "Polygon" geometry object
    /// as defined in IETF RFC 7946, for which the "type" member shall be set to the value "Polygon".
    area: Option<Polygon>,
}

/// Characteristics of the application, used in the ApplicationsList message.
/// The application characteristics relate to the system resources consumed by the application.
/// A device application can use this information e.g. for estimating
/// the cost of use of the application or for the expected user experience.
#[derive(Serialize, Deserialize, Clone)]
pub struct AppCharcs {
    /// The maximum size in Mbytes of the memory resource expected to be used
    /// by the MEC application instance in the MEC system.
    memory: Option<u32>,
    /// The maximum size in Mbytes of the storage resource expected to be used
    /// by the MEC application instance in the MEC system.
    storage: Option<u32>,
    /// The target round trip time in milliseconds supported by the MEC system
    /// for the MEC application instance.
    latency: Option<u32>,
    /// The required connection bandwidth in kbit/s for the use of the MEC application instance.
    bandwidth: Option<u32>,
    /// Required service continuity mode for this application. Permitted values:
    ///   0 = SERVICE_CONTINUITY_NOT_REQUIRED
    ///   1 = SERVICE_CONTINUITY_REQUIRED
    serviceCont: Option<u32>,
}

/// appInfo field used in the ApplicationList message
#[derive(Serialize, Deserialize, Clone)]
pub struct AppInfo {
    /// Identifier of this MEC application descriptor.
    /// It is equivalent to the appDId defined in clause 6.2.1.2 of ETSI GS MEC 010-2 [1].
    /// This attribute shall be globally unique.
    appDId: String,
    /// Name of the MEC application.
    /// The length of the value shall not exceed 32 characters.
    appName: String,
    /// Provider of the MEC application.
    /// The length of the value shall not exceed 32 characters.
    appProvider: String,
    /// Software version of the MEC application.
    /// The length of the value shall not exceed 32 characters.
    appSoftVersion: String,
    /// Identifies the version of the application descriptor.
    /// It is equivalent to the appDVersion defined in clause 6.2.1.2 of ETSI GS MEC 010-2
    appDVersion: String,
    /// Human readable description of the MEC application.
    /// The length of the value shall not exceed 128 characters.
    appDescription: String,
    /// Identifies the locations of the MEC application.
    appLocation: Vec<LocationConstraints>,
    /// Characteristics of the application.
    appCharcs: Option<AppCharcs>,
}

/// Extension for vendor specific information, used in the ApplicationsList message.
#[derive(Serialize, Deserialize, Clone)]
pub struct VendorSpecificExt {
    /// Vendor identifier.
    /// The length of the value shall not exceed 32 characters.
    /// The rest of the structure of vendor specific extension is not defined.
    vendorId: String,
}

impl VendorSpecificExt {
    fn empty() -> Self {
        Self {
            vendorId: "".to_owned(),
        }
    }
}

/// Inline structurre in the ApplicationList message.
#[derive(Serialize, Deserialize, Clone)]
pub struct AppList {
    /// Application information.
    appInfo: AppInfo,
    /// Extension for vendor specific information.
    vendorSpecificExt: Option<VendorSpecificExt>,
}

/// ApplicationList message used to retrieve the apps from the LCM proxy
#[derive(Serialize, Deserialize, Clone)]
pub struct ApplicationList {
    /// List of user applications available to the device application.
    pub appList: Vec<AppList>,
}

/// URI query parameters supported by the GET method on:
/// ```text
///     {apiRoot}/dev_app/v1/app_list
/// ```
/// The value of the attribute of the type String shall not exceed the length of 32 characters.
/// All the String values may contain multiple comma-separated values.
#[derive(Deserialize)]
pub struct ApplicationListInfo {
    /// Name to identify the MEC application.
    appName: Option<String>,
    /// Provider of the MEC application.
    appProvider: Option<String>,
    /// Software version of the MEC application.
    appSoftVersion: Option<String>,
    /// Required service continuity mode for this application.
    /// Permitted values:
    /// 0 = SERVICE_CONTINUITY_NOT_REQUIRED.
    /// 1 = SERVICE_CONTINUITY_REQUIRED.
    serviceCont: Option<u32>,
    /// Vendor identifier.
    vendorId: Option<String>,
}

impl ApplicationListInfo {
    pub fn empty() -> Self {
        Self {
            appName: None,
            appProvider: None,
            appSoftVersion: None,
            serviceCont: None,
            vendorId: None,
        }
    }

    fn to_hash_set(v: &Option<String>) -> HashSet<String> {
        let mut h = HashSet::new();
        if let Some(x) = v {
            x.split(",").for_each(|s| {
                h.insert(s.to_string());
            });
        }
        h
    }

    /// Return the appName elements, if any.
    pub fn app_names(&self) -> HashSet<String> {
        Self::to_hash_set(&self.appName)
    }

    /// Return the appProvider elements, if any.
    pub fn app_providers(&self) -> HashSet<String> {
        Self::to_hash_set(&self.appProvider)
    }

    /// Return the appSoftVersion elements, if any.
    pub fn app_soft_versions(&self) -> HashSet<String> {
        Self::to_hash_set(&self.appSoftVersion)
    }

    /// Return the vendorId elements, if any.
    pub fn vendor_ids(&self) -> HashSet<String> {
        Self::to_hash_set(&self.vendorId)
    }
}

impl ApplicationList {
    /// Return an empty list of applications.
    pub fn empty() -> Self {
        Self { appList: vec![] }
    }

    /// Return the AppList matching a given ApplicationListInfo.
    pub fn matching_info(&self, info: &ApplicationListInfo) -> Vec<AppList> {
        let mut ret: Vec<AppList> = vec![];

        // Retrieve all the sets of matching criteria.
        let app_names = info.app_names();
        let app_providers = info.app_providers();
        let app_soft_versions = info.app_soft_versions();
        let vendor_ids = info.vendor_ids();

        // Perform filtering.
        self.appList.iter().for_each(|x| {
            let empty = VendorSpecificExt::empty();
            if (app_names.is_empty() || app_names.contains(&x.appInfo.appName))
                && (app_providers.is_empty() || app_providers.contains(&x.appInfo.appProvider))
                && (app_soft_versions.is_empty()
                    || app_soft_versions.contains(&x.appInfo.appSoftVersion))
                && match info.serviceCont {
                    Some(y) => match &x.appInfo.appCharcs {
                        Some(app_charcs) => info.serviceCont == app_charcs.serviceCont,
                        None => false,
                    },
                    None => true,
                }
                && (vendor_ids.is_empty()
                    || vendor_ids.contains(
                        &x.vendorSpecificExt
                            .as_ref()
                            .unwrap_or_else(|| &empty)
                            .vendorId,
                    ))
            {
                ret.push(x.clone());
            }
        });

        ret
    }
}

impl Validate for ProblemDetails {}

impl Validate for Polygon {
    fn validate(&self) -> Result<(), String> {
        for polygon in &self.coordinates {
            for point in polygon {
                if point.len() != 2 {
                    return Err("each point must be identified by two values".to_string());
                }
            }
        }

        Ok(())
    }
}

impl Validate for CivicAddressElement {
    fn validate(&self) -> Result<(), String> {
        if self.caValue.is_empty() {
            return Err("Empty caValue in civicAddressElement".to_string());
        }
        Ok(())
    }
}

impl Validate for LocationConstraints {
    fn validate(&self) -> Result<(), String> {
        match &self.area {
            Some(polygon) => {
                if self.countryCode.is_some() || !self.civicAddressElement.is_empty() {
                    return Err(
                        "countryCode and civicAddressElement must be empty with area".to_string(),
                    );
                }
                polygon.validate()
            }
            None => {
                if self.countryCode.is_none() || self.countryCode == Some(String::from("")) {
                    Err("Empty countryCode in LocalConstraints".to_string())
                } else if self.civicAddressElement.is_empty() {
                    Err("Empty civicAddressElement in LocalConstraints".to_string())
                } else {
                    for c in &self.civicAddressElement {
                        let v = c.validate();
                        if v.is_err() {
                            return v;
                        }
                    }
                    Ok(())
                }
            }
        }
    }
}

impl Validate for AppCharcs {
    fn validate(&self) -> Result<(), String> {
        match &self.serviceCont {
            Some(x) => match x {
                0 | 1 => Ok(()),
                other => Err(format!("invalid serviceCont value: {other}")),
            },
            None => Ok(()),
        }
    }
}

impl Validate for AppInfo {
    fn validate(&self) -> Result<(), String> {
        let mut problems: Vec<String> = vec![];
        if self.appName.len() > 32 {
            problems.push("appName is too long".to_string());
        }
        if self.appProvider.len() > 32 {
            problems.push("appProvider is too long".to_string());
        }
        if self.appSoftVersion.len() > 32 {
            problems.push("appSoftVersion is too long".to_string());
        }
        if self.appDescription.len() > 128 {
            problems.push("appDescription is too long".to_string());
        }
        for c in &self.appLocation {
            add_problem(c, &mut problems);
        }
        match &self.appCharcs {
            Some(appCharcs) => add_problem(appCharcs, &mut problems),
            None => (),
        }

        check(problems)
    }
}

impl Validate for VendorSpecificExt {
    fn validate(&self) -> Result<(), String> {
        if self.vendorId.len() > 32 {
            Err("vendorId is too long".to_string())
        } else {
            Ok(())
        }
    }
}

impl Validate for AppList {
    fn validate(&self) -> Result<(), String> {
        let mut problems: Vec<String> = vec![];
        add_problem(&self.appInfo, &mut problems);
        match &self.vendorSpecificExt {
            Some(x) => add_problem(x, &mut problems),
            None => (),
        }
        check(problems)
    }
}

impl Validate for ApplicationList {
    fn validate(&self) -> Result<(), String> {
        let mut problems: Vec<String> = vec![];
        for a in &self.appList {
            add_problem(a, &mut problems);
        }
        check(problems)
    }
}

fn service_cont_valid(s: Option<u32>) -> bool {
    match s {
        Some(x) => match x {
            0 | 1 => true,
            _ => false,
        },
        None => true,
    }
}

fn value_or_not_specified(s: &Option<String>) -> &str {
    match s {
        Some(x) => x.as_str(),
        None => "not specified",
    }
}

impl Validate for ApplicationListInfo {
    fn validate(&self) -> Result<(), String> {
        let mut valid = true;
        self.app_names().iter().for_each(|x| valid &= x.len() <= 32);
        self.app_providers()
            .iter()
            .for_each(|x| valid &= x.len() <= 32);
        self.app_soft_versions()
            .iter()
            .for_each(|x| valid &= x.len() <= 32);
        valid &= service_cont_valid(self.serviceCont);
        self.vendor_ids()
            .iter()
            .for_each(|x| valid &= x.len() <= 32);
        match valid {
            true => Ok(()),
            false => Err("invalid query".to_string()),
        }
    }
}

fn service_cont_to_string(s: Option<u32>) -> String {
    match s {
        Some(x) => match x {
            0 => "not required".to_string(),
            1 => "required".to_string(),
            _other => "invalid value".to_string(),
        },
        None => "not specified".to_string(),
    }
}

impl Display for ProblemDetails {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({})", &self.status, &self.detail)
    }
}

impl Display for Polygon {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut areas: Vec<String> = vec![];
        for polygon in &self.coordinates {
            let mut points: Vec<String> = vec![];
            for point in polygon {
                let values: Vec<String> = point.iter().map(|x| x.to_string()).collect();
                points.push(format!("({})", values.join(",")));
            }
            areas.push(format!("[{}]", points.join(",")));
        }
        write!(f, "{}", areas.join(","))
    }
}

impl Display for CivicAddressElement {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.caType, self.caValue)
    }
}

impl Display for LocationConstraints {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self.area {
            Some(x) => write!(f, "area: {}", x),
            None => {
                let civics: Vec<String> = self
                    .civicAddressElement
                    .iter()
                    .map(|x| x.to_string())
                    .collect();
                write!(
                    f,
                    "country: {}, civic addresses: {}",
                    match &self.countryCode {
                        Some(x) => x.as_str(),
                        None => "not-present",
                    },
                    civics.join(",")
                )
            }
        }
    }
}

impl Display for AppCharcs {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "memory: {} MB, storage: {} MB, latency: {} ms, bandwidth: {} kb/s, continuity {}",
            &self.memory.unwrap_or(0),
            &self.storage.unwrap_or(0),
            &self.latency.unwrap_or(0),
            &self.bandwidth.unwrap_or(0),
            service_cont_to_string(self.serviceCont)
        )
    }
}

impl Display for AppInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let location_constraints: Vec<String> =
            self.appLocation.iter().map(|x| x.to_string()).collect();
        write!(f, "appDId: {}, appName: {}, appProvider: {}, appSoftVersion: {}, appDVersion: {}, appDescription: {}, appLocation: {}, appCharcs: {}",
    self.appDId, self.appName, self.appProvider, self.appSoftVersion, self.appDVersion, self.appDescription,
    location_constraints.join(","), match &self.appCharcs {
        Some(x) => x.to_string(),
        None => "unspecified".to_string(),
   })
    }
}

impl AppInfo {
    fn empty() -> Self {
        Self {
            appDId: "".to_owned(),
            appName: "".to_owned(),
            appProvider: "".to_owned(),
            appSoftVersion: "".to_owned(),
            appDVersion: "".to_owned(),
            appDescription: "".to_owned(),
            appLocation: vec![],
            appCharcs: None,
        }
    }
}

impl Display for VendorSpecificExt {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "vendorId: {}", self.vendorId)
    }
}

impl Display for AppList {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "appInfo: {}{}",
            self.appInfo,
            match &self.vendorSpecificExt {
                Some(x) => format!(", vendorSpecificExt: {}", x.vendorId),
                None => "".to_string(),
            }
        )
    }
}

impl Display for ApplicationList {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let apps: Vec<String> = self.appList.iter().map(|x| x.to_string()).collect();
        write!(f, "{}", apps.join("\n"))
    }
}

impl Display for ApplicationListInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "appName: {}, appProvider: {}, appSoftVersion: {}, serviceCont: {}, vendorId: {}",
            value_or_not_specified(&self.appName),
            value_or_not_specified(&self.appProvider),
            value_or_not_specified(&self.appSoftVersion),
            service_cont_to_string(self.serviceCont),
            value_or_not_specified(&self.vendorId)
        )
    }
}

pub fn application_list_from_file(file: &mut File) -> std::io::Result<ApplicationList> {
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    let j: ApplicationList = serde_json::from_str(content.as_str())?;
    Ok(j)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_polygon() -> Polygon {
        Polygon {
            coordinates: vec![
                vec![vec![0.0, 1.0], vec![1.0, 1.0], vec![1.0, 0.0]],
                vec![vec![0.0, 0.1], vec![0.1, 0.1], vec![0.1, 0.0]],
            ],
        }
    }

    fn default_app_charcs() -> AppCharcs {
        AppCharcs {
            memory: Some(100),
            storage: Some(200),
            latency: Some(50),
            bandwidth: Some(42),
            serviceCont: Some(1),
        }
    }

    fn default_app_info() -> AppInfo {
        AppInfo {
            appDId: "test_appDId".to_owned(),
            appName: "test_appName".to_owned(),
            appProvider: "test_appProvider".to_owned(),
            appSoftVersion: "test_appSoftVersion".to_owned(),
            appDVersion: "test_appDVersion".to_owned(),
            appDescription: "test_appDescription".to_owned(),
            appLocation: vec![LocationConstraints {
                countryCode: None,
                civicAddressElement: vec![],
                area: Some(default_polygon()),
            }],
            appCharcs: Some(default_app_charcs()),
        }
    }

    #[test]
    fn test_message_polygon() {
        let mut polygon = default_polygon();
        println!("{}", polygon);
        assert_eq!(Ok(()), polygon.validate());

        polygon.coordinates[0][0].push(2.0);
        assert!(polygon.validate().is_err());
    }

    #[test]
    fn test_message_civic_address_element() {
        let mut c = CivicAddressElement {
            caType: 0,
            caValue: "anything".to_owned(),
        };
        assert_eq!(Ok(()), c.validate());

        c.caValue.clear();
        assert!(c.validate().is_err());
    }

    #[test]
    fn test_message_location_constraints() {
        let mut c = LocationConstraints {
            countryCode: Some(String::from("it")),
            civicAddressElement: vec![CivicAddressElement {
                caType: 0,
                caValue: "anything".to_owned(),
            }],
            area: None,
        };
        assert_eq!(Ok(()), c.validate());
        println!("{}", c);

        c.area = Some(default_polygon());
        assert!(c.validate().is_err());

        c.countryCode = None;
        assert!(c.validate().is_err());

        c.civicAddressElement.clear();
        assert_eq!(Ok(()), c.validate());
        println!("{}", c);
    }

    #[test]
    fn test_message_app_charcs() {
        let a = AppCharcs {
            memory: None,
            storage: None,
            latency: None,
            bandwidth: None,
            serviceCont: None,
        };
        assert_eq!(Ok(()), a.validate());
        println!("{}", a);

        let mut a = default_app_charcs();
        assert_eq!(Ok(()), a.validate());
        println!("{}", a);

        a.serviceCont = Some(0);
        assert_eq!(Ok(()), a.validate());
        println!("{}", a);

        a.serviceCont = Some(2);
        assert!(a.validate().is_err());
    }

    #[test]
    fn test_message_app_info() {
        let a = AppInfo::empty();
        assert_eq!(Ok(()), a.validate());
        println!("{}", a);

        let mut a = default_app_info();
        assert_eq!(Ok(()), a.validate());
        println!("{}", a);

        let mut long = "".to_string();
        (0..33).for_each(|_| long.push('a'));
        a.appName = long;
        assert!(a.validate().is_err());
    }

    #[test]
    fn test_message_vendor_specific_ext() {
        let mut v = VendorSpecificExt {
            vendorId: "specific-extension".to_owned(),
        };
        assert_eq!(Ok(()), v.validate());
        println!("{}", v);

        let mut long = "".to_string();
        (0..33).for_each(|_| long.push('a'));
        v.vendorId = long;
        assert!(v.validate().is_err());
    }

    #[test]
    fn test_message_application_list() {
        let a = ApplicationList {
            appList: vec![
                AppList {
                    appInfo: default_app_info(),
                    vendorSpecificExt: None,
                },
                AppList {
                    appInfo: AppInfo::empty(),
                    vendorSpecificExt: Some(VendorSpecificExt {
                        vendorId: "vendor-specific".to_string(),
                    }),
                },
            ],
        };
        assert_eq!(Ok(()), a.validate());
        println!("{}", a);
    }

    #[test]
    fn test_application_list_info() {
        let info = ApplicationListInfo::empty();
        assert_eq!(Ok(()), info.validate());
        println!("{}", info);

        let mut info = ApplicationListInfo {
            appName: Some("app1,app2".to_string()),
            appProvider: Some("provider".to_string()),
            appSoftVersion: Some("1.0,2.0".to_string()),
            serviceCont: Some(1),
            vendorId: None,
        };
        assert_eq!(Ok(()), info.validate());
        println!("{}", info);

        let mut long = "".to_string();
        (0..33).for_each(|_| long.push('a'));
        info.vendorId = Some(long);
        assert!(info.validate().is_err());
    }

    #[test]
    fn test_message_problem_details() {
        let p = ProblemDetails {
            status: 401,
            detail: "not authorized".to_owned(),
        };
        assert_eq!(Ok(()), p.validate());
        println!("{}", p);
    }

    #[test]
    #[ignore]
    fn test_message_application_list_to_json() {
        let a = ApplicationList {
            appList: vec![AppList {
                appInfo: default_app_info(),
                vendorSpecificExt: None,
            }],
        };

        let filename = "application_list.json";
        if Path::new(filename).exists() {
            println!("will not overwrite: {}", filename);
            return;
        }
        let mut f = File::create(filename).expect("could not create file");
        let j = serde_json::to_string(&a).expect("could not serialize");
        f.write(j.as_bytes()).expect("could not write to file");
        println!("written:\n{}", a);
    }

    #[test]
    fn test_simple_ser_de() {
        #[derive(Serialize, Deserialize)]
        struct Field {
            name: String,
            nickname: Option<String>,
            age: u8,
        }
        #[derive(Serialize, Deserialize)]
        struct ExampleMessage {
            mtype: String,
            fields: Vec<Field>,
        }

        impl Display for Field {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                let nickname = match &self.nickname {
                    Some(x) => format!(" a.k.a. {}", x),
                    None => "".to_string(),
                };
                write!(f, "{}{nickname} (age {})", self.name, self.age)
            }
        }

        impl Display for ExampleMessage {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                let mut fields = String::from("");
                self.fields
                    .iter()
                    .for_each(|x| fields.push_str(format!("\n{}", x.to_string()).as_str()));
                write!(f, "type {}, fields: {}", self.mtype, fields)
            }
        }

        let msg_in = json!({
            "mtype": "example",
            "fields": [
                { "name": "Mickey Mouse", "age": 40, "nickname": "Mickey" },
                { "name": "Goofy",  "age": 45, },
            ]
        });

        let msg_ser = msg_in.to_string();
        println!("original:\n{}\n", msg_ser);
        let msg_out: ExampleMessage =
            serde_json::from_str(&msg_ser.to_string()).expect("could not deserialize");
        println!("structure:\n{}\n", msg_out.to_string());
        println!(
            "serialized:\n{}\n",
            serde_json::to_string(&msg_out).expect("could not serialize")
        );
    }
}
