//! Integration tests for the RBAC module

use no_downtime_service::rbac::{Permission, RbacState, Role};

#[test]
fn test_rbac_initialization() {
    let _rbac_state = RbacState::new();
    // We can't directly access the roles field, but we can test the functionality
    assert!(true); // Placeholder - we'll test functionality instead
}

#[test]
fn test_permission_creation() {
    let permission = Permission::new("test_permission");
    assert_eq!(permission.0, "test_permission");
}

#[test]
fn test_role_creation() {
    let role = Role {
        name: "test_role".to_string(),
        permissions: vec![Permission::new("read"), Permission::new("write")],
    };
    
    assert_eq!(role.name, "test_role");
    assert_eq!(role.permissions.len(), 2);
    assert!(role.permissions.contains(&Permission::new("read")));
    assert!(role.permissions.contains(&Permission::new("write")));
}

#[test]
fn test_rbac_state_default_roles() {
    let rbac_state = RbacState::new();
    
    // Check admin role permissions using has_permission method
    assert!(rbac_state.has_permission(&["admin".to_string()], &Permission::new("read")));
    assert!(rbac_state.has_permission(&["admin".to_string()], &Permission::new("write")));
    assert!(rbac_state.has_permission(&["admin".to_string()], &Permission::new("delete")));
    assert!(rbac_state.has_permission(&["admin".to_string()], &Permission::new("manage_users")));
    
    // Check user role permissions
    assert!(rbac_state.has_permission(&["user".to_string()], &Permission::new("read")));
    assert!(rbac_state.has_permission(&["user".to_string()], &Permission::new("write")));
    assert!(!rbac_state.has_permission(&["user".to_string()], &Permission::new("delete")));
    assert!(!rbac_state.has_permission(&["user".to_string()], &Permission::new("manage_users")));
    
    // Check guest role permissions
    assert!(rbac_state.has_permission(&["guest".to_string()], &Permission::new("read")));
    assert!(!rbac_state.has_permission(&["guest".to_string()], &Permission::new("write")));
    assert!(!rbac_state.has_permission(&["guest".to_string()], &Permission::new("delete")));
    assert!(!rbac_state.has_permission(&["guest".to_string()], &Permission::new("manage_users")));
}

#[test]
fn test_has_permission() {
    let rbac_state = RbacState::new();
    
    // Test with admin role
    assert!(rbac_state.has_permission(&["admin".to_string()], &Permission::new("read")));
    assert!(rbac_state.has_permission(&["admin".to_string()], &Permission::new("write")));
    assert!(rbac_state.has_permission(&["admin".to_string()], &Permission::new("delete")));
    assert!(rbac_state.has_permission(&["admin".to_string()], &Permission::new("manage_users")));
    
    // Test with user role
    assert!(rbac_state.has_permission(&["user".to_string()], &Permission::new("read")));
    assert!(rbac_state.has_permission(&["user".to_string()], &Permission::new("write")));
    assert!(!rbac_state.has_permission(&["user".to_string()], &Permission::new("delete")));
    assert!(!rbac_state.has_permission(&["user".to_string()], &Permission::new("manage_users")));
    
    // Test with guest role
    assert!(rbac_state.has_permission(&["guest".to_string()], &Permission::new("read")));
    assert!(!rbac_state.has_permission(&["guest".to_string()], &Permission::new("write")));
    assert!(!rbac_state.has_permission(&["guest".to_string()], &Permission::new("delete")));
    assert!(!rbac_state.has_permission(&["guest".to_string()], &Permission::new("manage_users")));
    
    // Test with multiple roles
    assert!(rbac_state.has_permission(&["user".to_string(), "admin".to_string()], &Permission::new("manage_users")));
}

#[test]
fn test_get_permissions() {
    let rbac_state = RbacState::new();
    
    // Test with admin role
    let admin_permissions = rbac_state.get_permissions(&["admin".to_string()]);
    assert!(admin_permissions.contains(&Permission::new("read")));
    assert!(admin_permissions.contains(&Permission::new("write")));
    assert!(admin_permissions.contains(&Permission::new("delete")));
    assert!(admin_permissions.contains(&Permission::new("manage_users")));
    
    // Test with user role
    let user_permissions = rbac_state.get_permissions(&["user".to_string()]);
    assert!(user_permissions.contains(&Permission::new("read")));
    assert!(user_permissions.contains(&Permission::new("write")));
    assert!(!user_permissions.contains(&Permission::new("delete")));
    assert!(!user_permissions.contains(&Permission::new("manage_users")));
    
    // Test with guest role
    let guest_permissions = rbac_state.get_permissions(&["guest".to_string()]);
    assert!(guest_permissions.contains(&Permission::new("read")));
    assert!(!guest_permissions.contains(&Permission::new("write")));
    assert!(!guest_permissions.contains(&Permission::new("delete")));
    assert!(!guest_permissions.contains(&Permission::new("manage_users")));
}

#[test]
fn test_add_role() {
    let mut rbac_state = RbacState::new();
    
    // Add a new role
    let moderator_role = Role {
        name: "moderator".to_string(),
        permissions: vec![
            Permission::new("read"),
            Permission::new("write"),
            Permission::new("moderate"),
        ],
    };
    
    rbac_state.add_role(moderator_role);
    
    // Check that the role has the correct permissions when checked
    assert!(rbac_state.has_permission(&["moderator".to_string()], &Permission::new("moderate")));
    assert!(rbac_state.has_permission(&["moderator".to_string()], &Permission::new("read")));
    assert!(rbac_state.has_permission(&["moderator".to_string()], &Permission::new("write")));
    assert!(!rbac_state.has_permission(&["moderator".to_string()], &Permission::new("delete")));
}

#[test]
fn test_permission_equality() {
    let permission1 = Permission::new("read");
    let permission2 = Permission::new("read");
    let permission3 = Permission::new("write");
    
    assert_eq!(permission1, permission2);
    assert_ne!(permission1, permission3);
}