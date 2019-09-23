use decscloud_codec as codec;
use guest::prelude::*;

pub(crate) const SYSTEMS_KEY: &str = "systems";
const SYSTEM_EXPIRATION_SECONDS: u32 = 60 * 5; // 5 minutes

// Functions in here right now use the raw KV capability
// TODO: they should be migrated to use the DECS capability once available

pub(crate) fn put_system(
    ctx: &CapabilitiesContext,
    system: &codec::systemmgr::System,
) -> guest::wapc::Result<()> {
    let system_json = serde_json::to_string(system)?;
    ctx.kv().list_add(SYSTEMS_KEY, &system.name)?;
    ctx.kv().set(
        &format!("system:{}", system.name),
        &system_json,
        Some(SYSTEM_EXPIRATION_SECONDS),
    )?;
    Ok(())
}

pub(crate) fn get_system(
    ctx: &CapabilitiesContext,
    system: &str,
) -> std::result::Result<http::Response, Box<dyn std::error::Error>> {
    get_system_details(ctx, system).map(|s| http::Response::json(s, 200, "OK"))
}

pub(crate) fn get_all_systems(
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

pub(crate) fn get_systems(ctx: &CapabilitiesContext) -> Result<Vec<String>> {
    ctx.kv().list_range(SYSTEMS_KEY, 0, 1000)
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

pub(crate) fn get_system_details(
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

/// Retrieves a list of all entities in a given shard that have a value for each
/// of the components indicated.
pub(crate) fn get_entities_for_component_set(_ctx: &CapabilitiesContext, _shard: &str, _components: &[String]) -> 
std::result::Result<Vec<String>, Box<dyn std::error::Error>> {
    // TODO - implement query
    // In redis this will look like SINTER decs:(shard):(component):entities
    Ok(vec![])
}
