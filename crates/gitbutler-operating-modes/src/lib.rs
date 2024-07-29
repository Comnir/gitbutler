use std::{fs, path::PathBuf};

use anyhow::{bail, Context, Result};
use gitbutler_command_context::CommandContext;
use serde::{Deserialize, Serialize};

/// Operating Modes:
/// Gitbutler currently has two main operating modes:
/// - `in workspace mode`: When the app is on the gitbutler/integration branch.
///     This is when normal operations can be performed.
/// - `outside workspace mode`: When the user has left the gitbutler/integration
///     branch to perform regular git commands.

const INTEGRATION_BRANCH_REF: &str = "refs/heads/gitbutler/integration";
const EDIT_BRANCH_REF: &str = "refs/heads/gitbutler/edit";

fn edit_mode_metadata_path(ctx: &CommandContext) -> PathBuf {
    ctx.project().gb_dir().join("edit_mode_metadata.toml")
}

fn read_edit_mode_metadata(ctx: &CommandContext) -> Result<EditModeMetadata> {
    let edit_mode_metadata = fs::read_to_string(edit_mode_metadata_path(ctx).as_path())
        .context("Failed to read edit mode metadata")?;

    toml::from_str(&edit_mode_metadata).context("Failed to parse edit mode metadata")
}

fn write_edit_mode_metadata(
    ctx: &CommandContext,
    edit_mode_metadata: &EditModeMetadata,
) -> Result<()> {
    let serialized_edit_mode_metadata =
        toml::to_string(edit_mode_metadata).context("Failed to serialize edit mode metadata")?;
    gitbutler_fs::write(
        edit_mode_metadata_path(ctx).as_path(),
        serialized_edit_mode_metadata,
    )
    .context("Failed to write edit mode metadata")?;

    Ok(())
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct EditModeMetadata {
    #[serde(with = "gitbutler_serde::serde::oid")]
    target_commit_sha: git2::Oid,
    source_branch: String,
}

#[derive(PartialEq)]
pub enum OperatingMode {
    /// The typical app state when its on the gitbutler/integration branch
    InWorkspace,
    /// When the user has chosen to leave the gitbutler/integration branch
    OutsideWorkspace,
    /// When the app is off of gitbutler/integration and in edit mode
    Edit(EditModeMetadata),
}

pub fn operating_mode(ctx: &CommandContext) -> Result<OperatingMode> {
    let head_ref = ctx.repository().head().context("failed to get head")?;
    let head_ref_name = head_ref.name().context("failed to get head name")?;

    if head_ref_name == INTEGRATION_BRANCH_REF {
        Ok(OperatingMode::InWorkspace)
    } else if head_ref_name == EDIT_BRANCH_REF {
        let edit_mode_metadata = read_edit_mode_metadata(ctx);

        match edit_mode_metadata {
            Ok(edit_mode_metadata) => Ok(OperatingMode::Edit(edit_mode_metadata)),
            Err(error) => {
                tracing::warn!(
                    "Failed to open in edit mode, falling back to outside workspace {}",
                    error
                );
                Ok(OperatingMode::OutsideWorkspace)
            }
        }
    } else {
        Ok(OperatingMode::OutsideWorkspace)
    }
}

pub fn in_open_workspace_mode(ctx: &CommandContext) -> Result<bool> {
    Ok(operating_mode(ctx)? == OperatingMode::InWorkspace)
}

pub fn assure_open_workspace_mode(ctx: &CommandContext) -> Result<()> {
    if in_open_workspace_mode(ctx)? {
        Ok(())
    } else {
        bail!("Expected to be in open workspace mode")
    }
}

pub fn in_edit_mode(ctx: &CommandContext) -> Result<bool> {
    match operating_mode(ctx)? {
        OperatingMode::Edit(_) => Ok(true),
        _ => Ok(false),
    }
}

pub fn assure_edit_mode(ctx: &CommandContext) -> Result<EditModeMetadata> {
    match operating_mode(ctx)? {
        OperatingMode::Edit(edit_mode_metadata) => Ok(edit_mode_metadata),
        _ => bail!("Expected to be in edit mode"),
    }
}

pub fn in_outside_workspace_mode(ctx: &CommandContext) -> Result<bool> {
    Ok(operating_mode(ctx)? == OperatingMode::OutsideWorkspace)
}

pub fn assure_outside_workspace_mode(ctx: &CommandContext) -> Result<()> {
    if in_outside_workspace_mode(ctx)? {
        Ok(())
    } else {
        bail!("Expected to be in outside workspace mode")
    }
}
