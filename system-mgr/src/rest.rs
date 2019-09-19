use crate::store;
use guest::prelude::*;

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
            QueryOperation::GetSystem(ref s) => store::get_system(ctx, s)?,
            QueryOperation::GetAllSystems => store::get_all_systems(ctx)?,
            QueryOperation::NotValid => http::Response::bad_request(),
        };
        Ok(protobytes(resp)?)
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
