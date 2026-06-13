// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Build target directory scanning and analysis
//!
//! Inspects `target/` directory for size, profiles, and stale artifacts.

use anyhow::{Context, Result};
use serde::Serialize;
use std::fs;
use std::path::PathBuf;

/// Target directory scanning
pub struct TargetScanning;

#[derive(Debug, Clone, Serialize)]
pub struct TargetInfo {
    pub path: PathBuf,
    pub total_size_gb: f64,
    pub profiles: Vec<ProfileBreakdown>,
    #[allow(dead_code)]
    pub stale_candidates: Vec<String>,
    pub configured_max_gb: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProfileBreakdown {
    pub profile: String,
    pub size_gb: f64,
    #[allow(dead_code)]
    pub file_count: usize,
}

impl TargetScanning {
    /// Scan target directory and return info
    pub fn scan(target_path: PathBuf) -> Result<TargetInfo> {
        if !target_path.exists() {
            return Ok(TargetInfo {
                path: target_path,
                total_size_gb: 0.0,
                profiles: vec![],
                stale_candidates: vec![],
                configured_max_gb: None,
            });
        }

        let total_size =
            Self::dir_size(&target_path).context("Failed to calculate target directory size")?;
        let total_size_gb = total_size as f64 / (1024.0 * 1024.0 * 1024.0);

        let profiles = Self::profile_breakdown(&target_path)?;
        let stale_candidates = Self::find_stale_artifacts(&target_path)?;

        Ok(TargetInfo {
            path: target_path,
            total_size_gb,
            profiles,
            stale_candidates,
            configured_max_gb: None,
        })
    }

    /// Get verdict for target size
    pub fn verdict(target_info: &TargetInfo, max_gb: f64) -> String {
        if target_info.total_size_gb < max_gb * 0.7 {
            "pass".to_string()
        } else if target_info.total_size_gb < max_gb {
            "warn".to_string()
        } else {
            "fail".to_string()
        }
    }

    fn dir_size(path: &PathBuf) -> Result<u64> {
        let mut size = 0u64;
        if path.is_dir() {
            for entry in fs::read_dir(path)? {
                let entry = entry?;
                let metadata = entry.metadata()?;
                if metadata.is_dir() {
                    size += Self::dir_size(&entry.path())?;
                } else {
                    size += metadata.len();
                }
            }
        }
        Ok(size)
    }

    fn profile_breakdown(target_path: &PathBuf) -> Result<Vec<ProfileBreakdown>> {
        let mut profiles = vec![];

        for profile in &["debug", "release", "bench"] {
            let profile_path = target_path.join(profile);
            if profile_path.exists() {
                let size = Self::dir_size(&profile_path)?;
                let file_count = Self::count_files(&profile_path)?;
                profiles.push(ProfileBreakdown {
                    profile: profile.to_string(),
                    size_gb: size as f64 / (1024.0 * 1024.0 * 1024.0),
                    file_count,
                });
            }
        }

        Ok(profiles)
    }

    fn count_files(path: &PathBuf) -> Result<usize> {
        let mut count = 0;
        if path.is_dir() {
            for entry in fs::read_dir(path)? {
                let entry = entry?;
                let metadata = entry.metadata()?;
                if metadata.is_dir() {
                    count += Self::count_files(&entry.path())?;
                } else {
                    count += 1;
                }
            }
        }
        Ok(count)
    }

    fn find_stale_artifacts(target_path: &PathBuf) -> Result<Vec<String>> {
        let mut stale = vec![];

        // Look for .dep-info files older than 7 days (example)
        if let Ok(entries) = fs::read_dir(target_path.join("debug")) {
            for entry in entries.flatten() {
                if let Ok(metadata) = entry.metadata() {
                    if let Ok(modified) = metadata.modified() {
                        if let Ok(duration) = std::time::SystemTime::now().duration_since(modified)
                        {
                            if duration.as_secs() > 7 * 24 * 3600 {
                                if let Some(name) = entry.file_name().to_str() {
                                    stale.push(format!("debug/{}", name));
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(stale)
    }
}
