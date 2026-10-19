use pumpkin_plugin_api::Context;
use pumpkin_plugin_api::permission::{Permission, PermissionDefault, PermissionLevel};

pub const LOGIN: &str = "CubicLogin:login";
pub const REGISTER: &str = "CubicLogin:register";
pub const ADMIN: &str = "CubicLogin:admin";

pub fn register(context: &Context) -> Result<(), String> {
    context.register_permission(&permission(
        LOGIN,
        "Allows a player to authenticate with CubicLogin",
        PermissionDefault::Allow,
    ))?;
    context.register_permission(&permission(
        REGISTER,
        "Allows a player to register a CubicLogin account",
        PermissionDefault::Allow,
    ))?;
    context.register_permission(&permission(
        ADMIN,
        "Allows management of CubicLogin accounts",
        PermissionDefault::Op(PermissionLevel::Four),
    ))?;

    Ok(())
}

fn permission(node: &str, description: &str, default: PermissionDefault) -> Permission {
    Permission {
        node: node.to_string(),
        description: description.to_string(),
        default,
        children: Vec::new(),
    }
}
