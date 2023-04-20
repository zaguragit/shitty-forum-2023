use crate::{data::{UserID, TopicID}, db::Permission};

use super::DB;

impl DB {
    pub fn is_admin(&self, user: &UserID) -> bool {
        let Some(permissions) = self.permissions.get(user) else {
            return false
        };
        permissions.iter().any(|x| matches!(x, Permission::Overlord))
    }
    pub fn grant_permission(&mut self, user: &UserID, permission: Permission) {
        if let Some(permissions) = self.permissions.get_mut(user) {
            permissions.push(permission);
        } else {
            self.permissions.insert(user.clone(), vec![permission]);
        }
    }
    pub fn revoke_permission(&mut self, user: &UserID, permission: Permission) -> bool {
        if let Some(permissions) = self.permissions.get_mut(user) {
            if let Some(p) = permissions.iter().position(|p| p == &permission) {
                permissions.remove(p);
                return true;
            }
        }
        false
    }
}