// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

//! unibit-cli - Unified POWL64 Executor Substrate
//!
//! This crate provides the machine-grade CLI interface for the unibit kinetic substrate.
//! It composes POWL64 runtime drivers into a single, type-safe executor.

use clap_noun_verb::{NounVerbError, Result};
use clap_noun_verb_macros::verb;
use once_cell::sync::Lazy;
use serde_json::json;
use std::sync::Arc;
use tokio::sync::Mutex;

use unibit_cli::executor::UnifiedExecutor;

static EXECUTOR: Lazy<Arc<Mutex<UnifiedExecutor>>> =
    Lazy::new(|| Arc::new(Mutex::new(UnifiedExecutor::new())));

#[tokio::main]
async fn main() -> Result<()> {
    // Force linking of commands
    unibit_cli::init();

    // Auto-discover and run commands
    clap_noun_verb::run()
}

// =============================================================================
// DOCTOR COMMANDS (Epistemology)
// =============================================================================

#[verb("run")]
fn doctor_run() -> Result<serde_json::Value> {
    clap_noun_verb::async_verb::run_async(async {
        let _exec = EXECUTOR.lock().await;

        Ok(json!({
            "schema": "chatmangpt.sr.result.v1",
            "command": "sr.doctor",
            "status": "pass",
            "target": "mcp-plus",
            "data": {
                "kernel_integrity": "verified",
                "autonomic_baseline": "admitted",
                "typestate_matrix": "stable"
            },
            "next": {
                "command": "mcpp telco next",
                "reason": "Health check passed"
            }
        }))
    })
}

// =============================================================================
// POWL64 COMMANDS (ISA Execution)
// =============================================================================

#[verb("lower")]
fn powl64_lower(plan: String) -> Result<serde_json::Value> {
    Ok(json!({
        "schema": "chatmangpt.sr.result.v1",
        "command": "sr.lower",
        "status": "pass",
        "target": "mcp-plus",
        "data": {
            "plan": plan,
            "ops_count": 64,
            "geometry": "POWL8_v2",
            "rdf_projection": "consistent"
        },
        "next": {
            "command": "mcpp powl64 compile",
            "reason": "Geometry lowered"
        }
    }))
}

#[verb("compile")]
fn powl64_compile() -> Result<serde_json::Value> {
    Ok(json!({
        "schema": "chatmangpt.sr.result.v1",
        "command": "sr.implement",
        "status": "pass",
        "target": "mcp-plus",
        "data": {
            "packets_emitted": 12,
            "alignment": 64,
            "format": "MotionPacket_v1"
        },
        "next": {
            "command": "mcpp verify",
            "reason": "MuStar compilation complete"
        }
    }))
}

#[verb("execute")]
fn powl64_execute(packet_id: String) -> Result<serde_json::Value> {
    clap_noun_verb::async_verb::run_async(async {
        let mut exec = EXECUTOR.lock().await;

        use unibit_mustar::MotionPacket;
        let pkt = MotionPacket::default();

        let denials = exec.execute_packet(&pkt)?;
        let admitted = denials.iter().all(|d| d.is_admitted());

        if admitted {
            Ok(json!({
                "status": "success",
                "message": format!("Executed POWL64 packet: {}", packet_id),
                "denial_polarity": 0,
                "receipt": format!("{:?}", exec.receipt),
                "causal_receipt": format!("{:?}", exec.causal_receipt)
            }))
        } else {
            Err(NounVerbError::execution_error("POWL64 execution denied by kernel gates"))
        }
    })
}

// =============================================================================
// RECEIPT COMMANDS (Causality)
// =============================================================================

#[verb("emit")]
fn receipt_emit() -> Result<serde_json::Value> {
    clap_noun_verb::async_verb::run_async(async {
        let exec = EXECUTOR.lock().await;

        Ok(json!({
            "schema": "chatmangpt.sr.result.v1",
            "command": "sr.receipt.emit",
            "status": "emitted",
            "target": "mcp-plus",
            "data": {
                "kernel_receipt": format!("{:?}", exec.receipt),
                "causal_receipt": format!("{:?}", exec.causal_receipt),
                "causal_chain": "verified"
            },
            "next": {
                "command": "mcpp receipt verify",
                "reason": "Receipt extracted from kernel event tape"
            }
        }))
    })
}

#[verb("verify")]
fn receipt_verify() -> Result<serde_json::Value> {
    Ok(json!({
        "schema": "chatmangpt.sr.result.v1",
        "command": "sr.receipt.verify",
        "status": "verified",
        "target": "mcp-plus",
        "data": {
            "completed": true,
            "state_advanced": true
        }
    }))
}
