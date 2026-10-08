use std::fs;

use zed_extension_api::{self as zed, settings::LspSettings, LanguageServerId, Result};

const REPO: &str = "dishmint/jaw";

struct JawExtension {
    cached_binary_path: Option<String>,
}

impl JawExtension {
    /// Resolve jaw-lsp: `lsp.jaw-lsp.binary.path` setting, then `PATH`, then
    /// the latest GitHub release (downloaded once into the extension's work dir).
    fn server_path(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<String> {
        if let Some(path) = LspSettings::for_worktree("jaw-lsp", worktree)
            .ok()
            .and_then(|s| s.binary)
            .and_then(|b| b.path)
        {
            return Ok(path);
        }

        if let Some(path) = worktree.which("jaw-lsp") {
            return Ok(path);
        }

        if let Some(path) = &self.cached_binary_path {
            if fs::metadata(path).is_ok_and(|m| m.is_file()) {
                return Ok(path.clone());
            }
        }

        let path = self.download(language_server_id)?;
        self.cached_binary_path = Some(path.clone());
        Ok(path)
    }

    fn download(&self, language_server_id: &LanguageServerId) -> Result<String> {
        zed::set_language_server_installation_status(
            language_server_id,
            &zed::LanguageServerInstallationStatus::CheckingForUpdate,
        );

        let release = zed::latest_github_release(
            REPO,
            zed::GithubReleaseOptions {
                require_assets: true,
                pre_release: false,
            },
        )?;

        let (os, arch) = zed::current_platform();
        let (target, file_type, exe) = match (os, arch) {
            (zed::Os::Mac, zed::Architecture::Aarch64) => {
                ("aarch64-apple-darwin", zed::DownloadedFileType::GzipTar, "jaw-lsp")
            }
            (zed::Os::Mac, zed::Architecture::X8664) => {
                ("x86_64-apple-darwin", zed::DownloadedFileType::GzipTar, "jaw-lsp")
            }
            (zed::Os::Linux, zed::Architecture::X8664) => {
                ("x86_64-unknown-linux-gnu", zed::DownloadedFileType::GzipTar, "jaw-lsp")
            }
            (zed::Os::Windows, zed::Architecture::X8664) => {
                ("x86_64-pc-windows-msvc", zed::DownloadedFileType::Zip, "jaw-lsp.exe")
            }
            _ => {
                return Err(
                    "no prebuilt jaw-lsp for this platform; build it with `cargo build --release -p jaw-lsp` and put it on PATH"
                        .into(),
                )
            }
        };

        let ext = match file_type {
            zed::DownloadedFileType::Zip => "zip",
            _ => "tar.gz",
        };
        let asset_name = format!("jaw-lsp-{target}.{ext}");
        let asset = release
            .assets
            .iter()
            .find(|a| a.name == asset_name)
            .ok_or_else(|| format!("release {} has no asset {asset_name}", release.version))?;

        let version_dir = format!("jaw-lsp-{}", release.version);
        let binary_path = format!("{version_dir}/{exe}");

        if !fs::metadata(&binary_path).is_ok_and(|m| m.is_file()) {
            zed::set_language_server_installation_status(
                language_server_id,
                &zed::LanguageServerInstallationStatus::Downloading,
            );
            zed::download_file(&asset.download_url, &version_dir, file_type)
                .map_err(|e| format!("failed to download {asset_name}: {e}"))?;
            zed::make_file_executable(&binary_path)?;

            // Drop older versions.
            if let Ok(entries) = fs::read_dir(".") {
                for entry in entries.flatten() {
                    let name = entry.file_name();
                    let name = name.to_string_lossy();
                    if name.starts_with("jaw-lsp-") && name != version_dir {
                        fs::remove_dir_all(entry.path()).ok();
                    }
                }
            }
        }

        Ok(binary_path)
    }
}

impl zed::Extension for JawExtension {
    fn new() -> Self {
        Self {
            cached_binary_path: None,
        }
    }

    fn language_server_command(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<zed::Command> {
        let command = self.server_path(language_server_id, worktree)?;
        let args = LspSettings::for_worktree("jaw-lsp", worktree)
            .ok()
            .and_then(|s| s.binary)
            .and_then(|b| b.arguments)
            .unwrap_or_default();

        Ok(zed::Command {
            command,
            args,
            env: worktree.shell_env(),
        })
    }
}

zed::register_extension!(JawExtension);
