use decscloud_common as codec;
use guest::prelude::*;

const USERS_KEY: &str = "decs:users";
pub(crate) const NOT_FOUND: &str = "Not found";

pub(crate) fn get_users(ctx: &CapabilitiesContext) -> Result<Vec<String>> {
    ctx.kv().list_range(USERS_KEY, 0, -1)
}

pub(crate) fn create_user(
    ctx: &CapabilitiesContext,
    user: &codec::users::User,
) -> std::result::Result<(usize, String), Box<dyn std::error::Error>> {
    let user_key = format!("decs:user:{}", user.id);
    let user_json = serde_json::to_string(&user)?;
    ctx.kv().set(&user_key, &user_json, None)?;
    ctx.kv().list_add(USERS_KEY, &user.id)?;

    let users = ctx.kv().list_range(USERS_KEY, 0, -1)?;
    match users.iter().position(|s| *s == user.id) {
        Some(p) => Ok((p, format!("decs.user.{}", user.id))),
        None => Err("item not in set".into()),
    }
}

/// Creates or sets a user. Returns a boolean indicating if the user previously existed
pub(crate) fn update_user(
    ctx: &CapabilitiesContext,
    user: &codec::users::User,
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    let user_key = format!("decs:user:{}", user.id);
    let user_json = serde_json::to_string(&user)?;
    ctx.kv().set(&user_key, &user_json, None)?;
    Ok(())
}

pub(crate) fn get_user_details(
    ctx: &CapabilitiesContext,
    rid: &str,
) -> std::result::Result<codec::users::User, Box<dyn std::error::Error>> {
    let user_key = rid.replace('.', ":");

    if let Some(v) = ctx.kv().get(&user_key)? {
        match serde_json::from_str::<codec::users::User>(&v) {
            Ok(r) => Ok(r),
            Err(e) => Err(e.into()),
        }
    } else {
        Err(NOT_FOUND.into())
    }
}
