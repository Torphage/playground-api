//! User-related authorization policies.

use crate::application::platform::authorization::Principal;
use crate::domain::platform::::values::{Permission, UserId};

/// Policy helpers for user-related actions.
pub struct UserPolicy;

impl UserPolicy {
    // /// Whether the principal may read the target user.
    // pub fn can_read(principal: &Principal, target_user_id: &UserId) -> bool {
    //     principal.user_id.as_uuid() == target_user_id.as_uuid()
    //         || principal.has(&Permission::identity_user_read())
    // }
    //
    // /// Whether the principal may update the target user.
    // pub fn can_update(principal: &Principal, target_user_id: &UserId) -> bool {
    //     principal.user_id.as_uuid() == target_user_id.as_uuid()
    //         || principal.has(&Permission::identity_user_update())
    // }
}
