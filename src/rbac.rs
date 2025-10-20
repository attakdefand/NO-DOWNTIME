//! Role-Based Access Control (RBAC) Module
//!
//! This module provides role-based access control functionality for the zero-downtime service.
//! It allows defining roles, permissions, and checking if a user has the required permissions.

use crate::auth::Claims;
use axum::{
    async_trait,
    extract::{FromRef, FromRequestParts},
    http::{request::Parts, StatusCode},
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Display;

/// Permission represents a specific action that can be performed
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Permission(pub String);

impl Permission {
    /// Create a new permission
    pub fn new(permission: &str) -> Self {
        Permission(permission.to_string())
    }
}

/// Role represents a collection of permissions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Role {
    /// Role name
    pub name: String,
    /// Permissions associated with this role
    pub permissions: Vec<Permission>,
}

/// RBAC Error
#[derive(Debug)]
pub enum RbacError {
    /// User does not have the required permission
    InsufficientPermissions,
    /// User not authenticated
    NotAuthenticated,
}

impl Display for RbacError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RbacError::InsufficientPermissions => write!(f, "Insufficient permissions"),
            RbacError::NotAuthenticated => write!(f, "Not authenticated"),
        }
    }
}

impl IntoResponse for RbacError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            RbacError::InsufficientPermissions => (StatusCode::FORBIDDEN, "Insufficient permissions"),
            RbacError::NotAuthenticated => (StatusCode::UNAUTHORIZED, "Not authenticated"),
        };

        (
            status,
            axum::Json(serde_json::json!({
                "error": error_message,
            })),
        )
            .into_response()
    }
}

/// RBAC State holds the role definitions
#[derive(Clone)]
pub struct RbacState {
    /// Role definitions
    roles: HashMap<String, Role>,
}

impl RbacState {
    /// Create a new RBAC state
    pub fn new() -> Self {
        let mut roles = HashMap::new();
        
        // Define default roles
        roles.insert(
            "admin".to_string(),
            Role {
                name: "admin".to_string(),
                permissions: vec![
                    Permission::new("read"),
                    Permission::new("write"),
                    Permission::new("delete"),
                    Permission::new("manage_users"),
                ],
            },
        );
        
        roles.insert(
            "user".to_string(),
            Role {
                name: "user".to_string(),
                permissions: vec![
                    Permission::new("read"),
                    Permission::new("write"),
                ],
            },
        );
        
        roles.insert(
            "guest".to_string(),
            Role {
                name: "guest".to_string(),
                permissions: vec![
                    Permission::new("read"),
                ],
            },
        );
        
        Self { roles }
    }
    
    /// Add a new role
    pub fn add_role(&mut self, role: Role) {
        self.roles.insert(role.name.clone(), role);
    }
    
    /// Check if a user with given roles has a specific permission
    pub fn has_permission(&self, user_roles: &[String], permission: &Permission) -> bool {
        user_roles.iter().any(|role_name| {
            if let Some(role) = self.roles.get(role_name) {
                role.permissions.contains(permission)
            } else {
                false
            }
        })
    }
    
    /// Get all permissions for a set of roles
    pub fn get_permissions(&self, user_roles: &[String]) -> Vec<Permission> {
        let mut permissions = Vec::new();
        
        for role_name in user_roles {
            if let Some(role) = self.roles.get(role_name) {
                for permission in &role.permissions {
                    if !permissions.contains(permission) {
                        permissions.push(permission.clone());
                    }
                }
            }
        }
        
        permissions
    }
}

impl Default for RbacState {
    fn default() -> Self {
        Self::new()
    }
}

/// RBAC extractor that checks if the authenticated user has the required permission
pub struct Rbac {
    /// User claims
    pub claims: Claims,
}

#[async_trait]
impl<S> FromRequestParts<S> for Rbac
where
    crate::auth::AuthState: FromRef<S>,
    RbacState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = RbacError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        // First, get the authenticated user
        let auth_user = crate::auth::AuthUser::from_request_parts(parts, state)
            .await
            .map_err(|_| RbacError::NotAuthenticated)?;
        
        Ok(Rbac {
            claims: auth_user.claims,
        })
    }
}

impl Rbac {
    /// Check if the user has a specific permission
    pub fn has_permission<S>(&self, state: &S, permission: &Permission) -> Result<(), RbacError>
    where
        RbacState: FromRef<S>,
    {
        let rbac_state = RbacState::from_ref(state);
        
        if rbac_state.has_permission(&self.claims.roles, permission) {
            Ok(())
        } else {
            Err(RbacError::InsufficientPermissions)
        }
    }
    
    /// Require a specific permission, returning an error if the user doesn't have it
    pub fn require_permission<S>(&self, state: &S, permission: &Permission) -> Result<(), RbacError>
    where
        RbacState: FromRef<S>,
    {
        self.has_permission(state, permission)
    }
}

/// RBAC extractor that requires a specific permission
pub struct RequirePermission {
    /// The required permission
    pub permission: Permission,
    /// User claims
    pub claims: Claims,
}

#[async_trait]
impl<S> FromRequestParts<S> for RequirePermission
where
    crate::auth::AuthState: FromRef<S>,
    RbacState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = RbacError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        // First, get the authenticated user
        let auth_user = crate::auth::AuthUser::from_request_parts(parts, state)
            .await
            .map_err(|_| RbacError::NotAuthenticated)?;
        
        // Get the required permission from the request extensions
        // In a real implementation, this would be determined by the route or handler
        // For now, we'll use a placeholder
        let permission = Permission::new("read");
        
        // Check if the user has the required permission
        let rbac_state = RbacState::from_ref(state);
        if !rbac_state.has_permission(&auth_user.claims.roles, &permission) {
            return Err(RbacError::InsufficientPermissions);
        }
        
        Ok(RequirePermission {
            permission,
            claims: auth_user.claims,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_permission_creation() {
        let permission = Permission::new("read");
        assert_eq!(permission.0, "read");
    }
    
    #[test]
    fn test_rbac_state_creation() {
        let rbac_state = RbacState::new();
        assert!(rbac_state.roles.contains_key("admin"));
        assert!(rbac_state.roles.contains_key("user"));
        assert!(rbac_state.roles.contains_key("guest"));
    }
    
    #[test]
    fn test_has_permission() {
        let rbac_state = RbacState::new();
        
        // Admin should have read permission
        assert!(rbac_state.has_permission(&["admin".to_string()], &Permission::new("read")));
        
        // User should have read permission
        assert!(rbac_state.has_permission(&["user".to_string()], &Permission::new("read")));
        
        // Guest should have read permission
        assert!(rbac_state.has_permission(&["guest".to_string()], &Permission::new("read")));
        
        // User should not have manage_users permission
        assert!(!rbac_state.has_permission(&["user".to_string()], &Permission::new("manage_users")));
        
        // Admin should have manage_users permission
        assert!(rbac_state.has_permission(&["admin".to_string()], &Permission::new("manage_users")));
    }
    
    #[test]
    fn test_get_permissions() {
        let rbac_state = RbacState::new();
        
        // Admin should have all permissions
        let admin_permissions = rbac_state.get_permissions(&["admin".to_string()]);
        assert!(admin_permissions.contains(&Permission::new("read")));
        assert!(admin_permissions.contains(&Permission::new("write")));
        assert!(admin_permissions.contains(&Permission::new("delete")));
        assert!(admin_permissions.contains(&Permission::new("manage_users")));
        
        // User should have read and write permissions
        let user_permissions = rbac_state.get_permissions(&["user".to_string()]);
        assert!(user_permissions.contains(&Permission::new("read")));
        assert!(user_permissions.contains(&Permission::new("write")));
        assert!(!user_permissions.contains(&Permission::new("delete")));
        assert!(!user_permissions.contains(&Permission::new("manage_users")));
    }
    
    #[test]
    fn test_add_role() {
        let mut rbac_state = RbacState::new();
        
        let moderator_role = Role {
            name: "moderator".to_string(),
            permissions: vec![
                Permission::new("read"),
                Permission::new("write"),
                Permission::new("moderate"),
            ],
        };
        
        rbac_state.add_role(moderator_role);
        
        // Moderator should have moderate permission
        assert!(rbac_state.has_permission(&["moderator".to_string()], &Permission::new("moderate")));
    }
}