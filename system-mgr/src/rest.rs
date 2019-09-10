use decscloud_codec as codec;
use guest::prelude::*;

pub(crate) const SYSTEMS_KEY: &str = "systems";
const SYSTEM_EXPIRATION_SECONDS: u32 = 60 * 5; // 5 minutes

pub fn handle_http(ctx: &CapabilitiesContext, req: impl Into<http::Request>) -> CallResult {
    let req = req.into();
    ctx.log(&format!("Handling HTTP request: {:?}", req));
    if req.method != "GET" {
        Ok(protobytes(http::Response {
            status_code: 405,
            status: "Method Not Allowed".to_string(),
            ..Default::default()
        })?)
    } else {
        let op = QueryOperation::from(req.path.as_str());
        let resp = match op {
            QueryOperation::GetSystem(ref s) => get_system(ctx, s)?,
            QueryOperation::GetAllSystems => get_all_systems(ctx)?,
            QueryOperation::NotValid => http::Response::bad_request(),
        };
        Ok(protobytes(resp)?)
    }
}

pub(crate) fn put_system(
    ctx: &CapabilitiesContext,
    system: &codec::systemmgr::System,
) -> Result<()> {
    let system_json = serde_json::to_string(system)?;
    ctx.kv().list_add(SYSTEMS_KEY, &system.name)?;
    ctx.kv().set(
        &format!("system:{}", system.name),
        &system_json,
        Some(SYSTEM_EXPIRATION_SECONDS),
    )?;
    Ok(())
}

fn get_system(
    ctx: &CapabilitiesContext,
    system: &str,
) -> std::result::Result<http::Response, Box<dyn std::error::Error>> {
    get_system_details(ctx, system).map(|s| http::Response::json(s, 200, "OK"))
}

fn get_all_systems(
    ctx: &CapabilitiesContext,
) -> std::result::Result<http::Response, Box<dyn std::error::Error>> {
    match ctx.kv().list_range(SYSTEMS_KEY, 0, 1000) {
        Ok(systems_list) => {
            get_system_list(ctx, systems_list).map(|s| http::Response::json(s, 200, "OK"))
        }
        Err(e) => Ok(http::Response::internal_server_error(&format!(
            "Failed to retrieve systems: {}",
            e
        ))),
    }
}

pub(crate) fn get_system_list(
    ctx: &CapabilitiesContext,
    systems: Vec<String>,
) -> std::result::Result<Vec<codec::systemmgr::System>, Box<dyn std::error::Error>> {
    systems
        .iter()
        .filter_map(|sys_id| match get_system_details(ctx, sys_id) {
            Ok(r) => Some(Ok(r)),
            Err(_) => {
                // Item details have expired, remove it from the list
                ctx.kv().list_del_item(SYSTEMS_KEY, sys_id).unwrap();
                None
            }
        })
        .collect()
}

fn get_system_details(
    ctx: &CapabilitiesContext,
    system: &str,
) -> std::result::Result<codec::systemmgr::System, Box<dyn std::error::Error>> {
    let systemkey = format!("system:{}", system);

    if let Some(v) = ctx.kv().get(&systemkey)? {
        match serde_json::from_str::<codec::systemmgr::System>(&v) {
            Ok(r) => Ok(r),
            Err(e) => Err(e.into()),
        }
    } else {
        Err("system doesn't exist".into())
    }
}

#[derive(Debug, PartialEq)]
enum QueryOperation {
    GetSystem(String),
    GetAllSystems,
    NotValid,
}

impl From<&str> for QueryOperation {
    fn from(source: &str) -> Self {
        if !source.contains("system") {
            Self::NotValid
        } else {
            let src = if source.starts_with('/') {
                source.chars().skip(1).collect::<String>()
            } else {
                source.to_string()
            };
            let parts: Vec<&str> = src.split('/').collect();
            if parts.len() == 1 && parts[0] == "systems" {
                Self::GetAllSystems
            } else if parts.len() == 2 && parts[0] == "system" {
                Self::GetSystem(parts[1].to_string())
            } else {
                Self::NotValid
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::QueryOperation;

    #[test]
    fn test_queryop_parse() {
        assert_eq!(
            QueryOperation::from("/system/system-12"),
            QueryOperation::GetSystem("system-12".to_string())
        );
        assert_eq!(
            QueryOperation::from("system/system-12"),
            QueryOperation::GetSystem("system-12".to_string())
        );

        assert_eq!(
            QueryOperation::from("systems"),
            QueryOperation::GetAllSystems
        );

        assert_eq!(
            QueryOperation::from("flooferdoof"),
            QueryOperation::NotValid
        );

        assert_eq!(
            QueryOperation::from("systems/system20"),
            QueryOperation::NotValid
        );
    }
}
