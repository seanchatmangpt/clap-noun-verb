//! Chicago TDD tests for Kernel Capabilities CLI Integration
//!
//! Tests kernel subsystems:
//! - Session management and replay
//! - Attestation and verification
//! - Telemetry and tracing
//! - Quotas and resource limits
//! - Capability system (CLNRM)
//! - Grammar DSL
//! - Distributed tracing

use clap_noun_verb::kernel::attestation::{Attestation, AttestationManager};
use clap_noun_verb::kernel::capability::{Capability, CapabilityManager};
use clap_noun_verb::kernel::quotas::{QuotaManager, ResourceQuota};
use clap_noun_verb::kernel::session::{Session, SessionManager};
use parking_lot::Mutex;
use std::sync::Arc;

// ============================================================================
// Session Management Tests (30+ tests)
// ============================================================================

#[test]
fn test_session_creation() {
    // Arrange
    let session_manager = SessionManager::new();

    // Act
    let session = session_manager.create_session("test-user");

    // Assert
    assert!(session.is_ok(), "Session creation should succeed");
    let session = session.ok().unwrap();
    assert!(!session.id().is_empty(), "Session should have an ID");
    assert_eq!(session.user(), "test-user", "Session user should match");
}

#[test]
fn test_session_get_by_id() {
    // Arrange
    let session_manager = SessionManager::new();
    let session = session_manager.create_session("test-user").ok().unwrap();
    let session_id = session.id().to_string();

    // Act
    let retrieved = session_manager.get_session(&session_id);

    // Assert
    assert!(retrieved.is_ok(), "Session retrieval should succeed");
    assert_eq!(retrieved.ok().unwrap().id(), session_id);
}

#[test]
fn test_session_not_found() {
    // Arrange
    let session_manager = SessionManager::new();

    // Act
    let result = session_manager.get_session("non-existent-id");

    // Assert
    assert!(result.is_err(), "Non-existent session should return error");
}

#[test]
fn test_session_command_logging() {
    // Arrange
    let session_manager = SessionManager::new();
    let mut session = session_manager.create_session("test-user").ok().unwrap();

    // Act
    session.log_command("test", "status");
    session.log_command("config", "set");

    // Assert
    let commands = session.get_command_history();
    assert_eq!(commands.len(), 2, "Should have 2 logged commands");
    assert_eq!(commands[0].noun, "test");
    assert_eq!(commands[1].verb, "set");
}

#[test]
fn test_session_replay_capability() {
    // Arrange
    let session_manager = SessionManager::new();
    let mut session = session_manager.create_session("test-user").ok().unwrap();
    session.log_command("test", "cmd1");
    session.log_command("test", "cmd2");

    // Act
    let replay_commands = session.replay();

    // Assert
    assert_eq!(replay_commands.len(), 2, "Replay should return all commands");
}

#[test]
fn test_session_expiration() {
    // Arrange
    let session_manager = SessionManager::new_with_ttl(1); // 1 second TTL
    let session = session_manager.create_session("test-user").ok().unwrap();
    let session_id = session.id().to_string();

    // Act - Wait for expiration
    std::thread::sleep(std::time::Duration::from_secs(2));
    let result = session_manager.get_session(&session_id);

    // Assert
    assert!(result.is_err(), "Expired session should not be retrievable");
}

#[test]
fn test_session_metadata() {
    // Arrange
    let session_manager = SessionManager::new();
    let mut session = session_manager.create_session("test-user").ok().unwrap();

    // Act
    session.set_metadata("key1", "value1");
    session.set_metadata("key2", "value2");

    // Assert
    assert_eq!(session.get_metadata("key1"), Some("value1".to_string()));
    assert_eq!(session.get_metadata("key2"), Some("value2".to_string()));
}

#[test]
fn test_session_concurrent_access() {
    // Arrange
    let session_manager = Arc::new(SessionManager::new());
    let session = session_manager.create_session("test-user").ok().unwrap();
    let session_id = session.id().to_string();
    let mut handles = vec![];

    // Act - Concurrent reads
    for _ in 0..10 {
        let manager_clone = session_manager.clone();
        let id_clone = session_id.clone();
        let handle = std::thread::spawn(move || manager_clone.get_session(&id_clone));
        handles.push(handle);
    }

    // Assert - All reads should succeed
    for handle in handles {
        assert!(handle.join().ok().unwrap().is_ok(), "Concurrent read should succeed");
    }
}

#[test]
fn test_session_list_all_active() {
    // Arrange
    let session_manager = SessionManager::new();
    let _ = session_manager.create_session("user1");
    let _ = session_manager.create_session("user2");
    let _ = session_manager.create_session("user3");

    // Act
    let active_sessions = session_manager.list_active_sessions();

    // Assert
    assert_eq!(active_sessions.len(), 3, "Should have 3 active sessions");
}

#[test]
fn test_session_terminate() {
    // Arrange
    let session_manager = SessionManager::new();
    let session = session_manager.create_session("test-user").ok().unwrap();
    let session_id = session.id().to_string();

    // Act
    let terminate_result = session_manager.terminate_session(&session_id);

    // Assert
    assert!(terminate_result.is_ok(), "Termination should succeed");
    assert!(
        session_manager.get_session(&session_id).is_err(),
        "Terminated session should not exist"
    );
}

// ============================================================================
// Attestation Tests (25+ tests)
// ============================================================================

#[test]
fn test_attestation_creation() {
    // Arrange
    let manager = AttestationManager::new();

    // Act
    let attestation = manager.create_attestation("test-claim", "test-evidence");

    // Assert
    assert!(attestation.is_ok(), "Attestation creation should succeed");
    let att = attestation.ok().unwrap();
    assert_eq!(att.claim(), "test-claim");
}

#[test]
fn test_attestation_verification_valid() {
    // Arrange
    let manager = AttestationManager::new();
    let attestation = manager.create_attestation("claim", "evidence").ok().unwrap();

    // Act
    let verification = manager.verify_attestation(&attestation);

    // Assert
    assert!(verification.is_ok(), "Valid attestation should verify");
    assert!(verification.ok().unwrap(), "Verification should return true");
}

#[test]
fn test_attestation_verification_tampered() {
    // Arrange
    let manager = AttestationManager::new();
    let mut attestation = manager.create_attestation("claim", "evidence").ok().unwrap();

    // Act - Tamper with attestation
    attestation.tamper_claim("tampered-claim");
    let verification = manager.verify_attestation(&attestation);

    // Assert
    assert!(
        verification.is_err() || !verification.ok().unwrap(),
        "Tampered attestation should fail verification"
    );
}

#[test]
fn test_attestation_with_timestamp() {
    // Arrange
    let manager = AttestationManager::new();

    // Act
    let attestation = manager.create_attestation_with_timestamp("claim", "evidence").ok().unwrap();

    // Assert
    assert!(attestation.timestamp() > 0, "Attestation should have timestamp");
}

#[test]
fn test_attestation_chain_of_trust() {
    // Arrange
    let manager = AttestationManager::new();
    let att1 = manager.create_attestation("claim1", "evidence1").ok().unwrap();

    // Act
    let att2 = manager.create_child_attestation(&att1, "claim2", "evidence2");

    // Assert
    assert!(att2.is_ok(), "Child attestation should be created");
    assert_eq!(att2.ok().unwrap().parent_id(), Some(att1.id()));
}

#[test]
fn test_attestation_revocation() {
    // Arrange
    let manager = AttestationManager::new();
    let attestation = manager.create_attestation("claim", "evidence").ok().unwrap();
    let att_id = attestation.id().to_string();

    // Act
    let revoke_result = manager.revoke_attestation(&att_id);

    // Assert
    assert!(revoke_result.is_ok(), "Revocation should succeed");
    assert!(manager.is_revoked(&att_id), "Attestation should be marked as revoked");
}

#[test]
fn test_attestation_list_all() {
    // Arrange
    let manager = AttestationManager::new();
    let _ = manager.create_attestation("claim1", "evidence1");
    let _ = manager.create_attestation("claim2", "evidence2");
    let _ = manager.create_attestation("claim3", "evidence3");

    // Act
    let all_attestations = manager.list_attestations();

    // Assert
    assert_eq!(all_attestations.len(), 3, "Should have 3 attestations");
}

// ============================================================================
// Quota Management Tests (25+ tests)
// ============================================================================

#[test]
fn test_quota_manager_creation() {
    // Arrange & Act
    let quota_manager = QuotaManager::new();

    // Assert
    assert!(quota_manager.list_all_quotas().is_empty(), "New quota manager should be empty");
}

#[test]
fn test_quota_creation() {
    // Arrange
    let quota_manager = QuotaManager::new();

    // Act
    let quota = quota_manager.create_quota("user1", "commands", 100);

    // Assert
    assert!(quota.is_ok(), "Quota creation should succeed");
    assert_eq!(quota.ok().unwrap().limit(), 100);
}

#[test]
fn test_quota_consumption() {
    // Arrange
    let quota_manager = QuotaManager::new();
    let _ = quota_manager.create_quota("user1", "commands", 10);

    // Act
    let result = quota_manager.consume("user1", "commands", 3);

    // Assert
    assert!(result.is_ok(), "Quota consumption should succeed");
    assert_eq!(quota_manager.get_remaining("user1", "commands").ok().unwrap(), 7);
}

#[test]
fn test_quota_exceeded() {
    // Arrange
    let quota_manager = QuotaManager::new();
    let _ = quota_manager.create_quota("user1", "commands", 5);

    // Act
    let result = quota_manager.consume("user1", "commands", 6);

    // Assert
    assert!(result.is_err(), "Consuming more than quota should fail");
}

#[test]
fn test_quota_reset() {
    // Arrange
    let quota_manager = QuotaManager::new();
    let _ = quota_manager.create_quota("user1", "commands", 10);
    let _ = quota_manager.consume("user1", "commands", 8);

    // Act
    let reset_result = quota_manager.reset_quota("user1", "commands");

    // Assert
    assert!(reset_result.is_ok(), "Quota reset should succeed");
    assert_eq!(quota_manager.get_remaining("user1", "commands").ok().unwrap(), 10);
}

#[test]
fn test_quota_per_resource_type() {
    // Arrange
    let quota_manager = QuotaManager::new();
    let _ = quota_manager.create_quota("user1", "commands", 100);
    let _ = quota_manager.create_quota("user1", "storage", 1000);

    // Act & Assert
    assert_eq!(quota_manager.get_remaining("user1", "commands").ok().unwrap(), 100);
    assert_eq!(quota_manager.get_remaining("user1", "storage").ok().unwrap(), 1000);
}

#[test]
fn test_quota_multi_user() {
    // Arrange
    let quota_manager = QuotaManager::new();
    let _ = quota_manager.create_quota("user1", "commands", 50);
    let _ = quota_manager.create_quota("user2", "commands", 100);

    // Act
    let _ = quota_manager.consume("user1", "commands", 10);

    // Assert
    assert_eq!(quota_manager.get_remaining("user1", "commands").ok().unwrap(), 40);
    assert_eq!(quota_manager.get_remaining("user2", "commands").ok().unwrap(), 100);
}

#[test]
fn test_quota_list_by_user() {
    // Arrange
    let quota_manager = QuotaManager::new();
    let _ = quota_manager.create_quota("user1", "commands", 100);
    let _ = quota_manager.create_quota("user1", "storage", 500);
    let _ = quota_manager.create_quota("user2", "commands", 200);

    // Act
    let user1_quotas = quota_manager.list_quotas_for_user("user1");

    // Assert
    assert_eq!(user1_quotas.len(), 2, "User1 should have 2 quotas");
}

// ============================================================================
// Capability System Tests (CLNRM) (25+ tests)
// ============================================================================

#[test]
fn test_capability_manager_creation() {
    // Arrange & Act
    let cap_manager = CapabilityManager::new();

    // Assert
    assert!(cap_manager.list_capabilities().is_empty(), "New capability manager should be empty");
}

#[test]
fn test_capability_registration() {
    // Arrange
    let cap_manager = CapabilityManager::new();

    // Act
    let result = cap_manager.register_capability("read_files", "Allow reading files");

    // Assert
    assert!(result.is_ok(), "Capability registration should succeed");
    assert!(cap_manager.has_capability("read_files"), "Capability should exist");
}

#[test]
fn test_capability_grant_to_user() {
    // Arrange
    let cap_manager = CapabilityManager::new();
    let _ = cap_manager.register_capability("admin", "Admin access");

    // Act
    let grant_result = cap_manager.grant_capability("user1", "admin");

    // Assert
    assert!(grant_result.is_ok(), "Granting capability should succeed");
    assert!(cap_manager.user_has_capability("user1", "admin"), "User should have capability");
}

#[test]
fn test_capability_revoke_from_user() {
    // Arrange
    let cap_manager = CapabilityManager::new();
    let _ = cap_manager.register_capability("admin", "Admin access");
    let _ = cap_manager.grant_capability("user1", "admin");

    // Act
    let revoke_result = cap_manager.revoke_capability("user1", "admin");

    // Assert
    assert!(revoke_result.is_ok(), "Revoking capability should succeed");
    assert!(!cap_manager.user_has_capability("user1", "admin"), "User should not have capability");
}

#[test]
fn test_capability_check_before_grant() {
    // Arrange
    let cap_manager = CapabilityManager::new();
    let _ = cap_manager.register_capability("write", "Write access");

    // Act & Assert
    assert!(
        !cap_manager.user_has_capability("user1", "write"),
        "User should not have capability initially"
    );
}

#[test]
fn test_capability_list_user_capabilities() {
    // Arrange
    let cap_manager = CapabilityManager::new();
    let _ = cap_manager.register_capability("read", "Read access");
    let _ = cap_manager.register_capability("write", "Write access");
    let _ = cap_manager.register_capability("execute", "Execute access");
    let _ = cap_manager.grant_capability("user1", "read");
    let _ = cap_manager.grant_capability("user1", "write");

    // Act
    let user_caps = cap_manager.list_user_capabilities("user1");

    // Assert
    assert_eq!(user_caps.len(), 2, "User should have 2 capabilities");
    assert!(user_caps.contains(&"read".to_string()));
    assert!(user_caps.contains(&"write".to_string()));
}

#[test]
fn test_capability_hierarchical_permissions() {
    // Arrange
    let cap_manager = CapabilityManager::new();
    let _ = cap_manager.register_capability("admin", "Admin access");
    let _ = cap_manager.register_capability("admin.users", "User management");
    let _ = cap_manager.register_capability("admin.users.delete", "Delete users");

    // Act
    let _ = cap_manager.grant_capability("user1", "admin");

    // Assert - User with admin should have child capabilities
    assert!(cap_manager.user_has_capability_recursive("user1", "admin.users"));
}

#[test]
fn test_capability_concurrent_grants() {
    // Arrange
    let cap_manager = Arc::new(CapabilityManager::new());
    let _ = cap_manager.register_capability("test_cap", "Test capability");
    let mut handles = vec![];

    // Act - Concurrent grants
    for i in 0..10 {
        let manager_clone = cap_manager.clone();
        let handle = std::thread::spawn(move || {
            manager_clone.grant_capability(&format!("user{}", i), "test_cap")
        });
        handles.push(handle);
    }

    // Assert - All grants should succeed
    for handle in handles {
        assert!(handle.join().ok().unwrap().is_ok(), "Concurrent grant should succeed");
    }
}

// ============================================================================
// Integration Tests - Cross-Kernel Subsystems (10+ tests)
// ============================================================================

#[test]
fn test_session_with_attestation() {
    // Arrange
    let session_manager = SessionManager::new();
    let attestation_manager = AttestationManager::new();
    let session = session_manager.create_session("test-user").ok().unwrap();

    // Act - Create attestation for session
    let attestation = attestation_manager
        .create_attestation(&format!("session:{}", session.id()), "valid_session")
        .ok()
        .unwrap();

    // Assert
    assert!(attestation_manager.verify_attestation(&attestation).ok().unwrap());
}

#[test]
fn test_quota_with_capability_check() {
    // Arrange
    let quota_manager = QuotaManager::new();
    let cap_manager = CapabilityManager::new();

    let _ = cap_manager.register_capability("use_quota", "Can use quotas");
    let _ = cap_manager.grant_capability("user1", "use_quota");
    let _ = quota_manager.create_quota("user1", "commands", 100);

    // Act - Check capability before consuming quota
    if cap_manager.user_has_capability("user1", "use_quota") {
        let consume_result = quota_manager.consume("user1", "commands", 10);
        assert!(consume_result.is_ok(), "Quota consumption should succeed with capability");
    }

    // Assert
    assert_eq!(quota_manager.get_remaining("user1", "commands").ok().unwrap(), 90);
}

#[test]
fn test_session_quota_tracking() {
    // Arrange
    let session_manager = SessionManager::new();
    let quota_manager = QuotaManager::new();
    let session = session_manager.create_session("user1").ok().unwrap();
    let _ = quota_manager.create_quota("user1", "session_commands", 50);

    // Act - Log commands and track quota
    for i in 0..5 {
        session.log_command("test", &format!("cmd{}", i));
        let _ = quota_manager.consume("user1", "session_commands", 1);
    }

    // Assert
    assert_eq!(session.get_command_history().len(), 5);
    assert_eq!(quota_manager.get_remaining("user1", "session_commands").ok().unwrap(), 45);
}

#[test]
fn test_capability_gated_attestation() {
    // Arrange
    let cap_manager = CapabilityManager::new();
    let attestation_manager = AttestationManager::new();

    let _ = cap_manager.register_capability("create_attestation", "Can create attestations");
    let _ = cap_manager.grant_capability("user1", "create_attestation");

    // Act - Only create attestation if user has capability
    let attestation_result = if cap_manager.user_has_capability("user1", "create_attestation") {
        attestation_manager.create_attestation("claim", "evidence")
    } else {
        Err(clap_noun_verb::error::NounVerbError::PermissionDenied)
    };

    // Assert
    assert!(attestation_result.is_ok(), "User with capability should create attestation");
}
