//! Capability commands - intent surface

use clap_noun_verb_macros::verb;
use clap_noun_verb::Result;
use crate::domain::capability::{Capability, CapabilityResolver, CapabilityInfo};
use crate::outputs::CapabilityEnabledOutput;

/// Enable a capability surface
///
/// Resolves the desired surface to pack/install/runtime consequences.
#[verb("enable")]
fn enable_capability(
    surface: String,
    projection: Option<String>,
    runtime: Option<String>,
    profile: Option<String>,
) -> Result<CapabilityEnabledOutput> {
    let resolver = CapabilityResolver::new();
    let capability = Capability {
        surface,
        projection,
        runtime,
        profile: profile.unwrap_or_else(|| "default".to_string()),
    };

    let resolution = resolver.resolve(&capability)?;

    Ok(CapabilityEnabledOutput {
        capability: format!("{}/{}/{}",
            resolution.capability.surface,
            resolution.capability.projection.as_deref().unwrap_or("none"),
            resolution.capability.runtime.as_deref().unwrap_or("none")
        ),
        packs_required: resolution.packs,
        install_actions: resolution.actions,
        validation_required: resolution.needs_policy_check,
    })
}

/// List available capabilities
#[verb("list")]
fn list_capabilities() -> Result<crate::outputs::CapabilityListOutput> {
    let capabilities = Capability::all_available()
        .into_iter()
        .map(|c| crate::outputs::CapabilityInfoItem {
            name: c.name,
            description: c.description,
        })
        .collect();

    Ok(crate::outputs::CapabilityListOutput {
        capabilities,
    })
}

/// Show capability details
#[verb("show")]
fn show_capability(name: String) -> Result<crate::outputs::CapabilityShowOutput> {
    let info = Capability::find(&name)?;
    Ok(crate::outputs::CapabilityShowOutput {
        name: info.name,
        description: info.description,
    })
}
