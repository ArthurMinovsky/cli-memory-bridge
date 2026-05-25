use std::{
    env, fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, ensure};
use hf_hub::{
    Cache,
    api::sync::ApiBuilder,
};

const DEFAULT_MODEL_REPO: &str = "minishlab/potion-multilingual-128M";
const DEFAULT_MODEL_DIRNAME: &str = "potion-multilingual-128M";
const REQUIRED_MODEL_FILES: [&str; 3] = ["tokenizer.json", "model.safetensors", "config.json"];

pub fn resolve_model_dir() -> Result<PathBuf> {
    let data_dir = configured_data_dir()?;
    let explicit = env::var_os("CLI_MEMORY_MODEL_PATH").map(PathBuf::from);
    let model_dir = resolved_model_dir_for(explicit, &data_dir);

    if env::var_os("CLI_MEMORY_MODEL_PATH").is_some() {
        return Ok(model_dir);
    }

    let hf_cache_dir = data_dir.join(".hf-cache");
    ensure_model_cached_with(
        &model_dir,
        &hf_cache_dir,
        env::var("HF_HUB_TOKEN").ok(),
        download_model_file,
    )?;
    Ok(model_dir)
}

pub fn current_model_dir() -> Result<PathBuf> {
    let data_dir = configured_data_dir()?;
    let explicit = env::var_os("CLI_MEMORY_MODEL_PATH").map(PathBuf::from);
    Ok(resolved_model_dir_for(explicit, &data_dir))
}

pub fn model_cache_ready() -> Result<bool> {
    Ok(required_model_files_present(&current_model_dir()?))
}

fn configured_data_dir() -> Result<PathBuf> {
    if let Some(path) = env::var_os("CLI_MEMORY_DATA_DIR") {
        return Ok(PathBuf::from(path));
    }

    let home = env::var_os("HOME")
        .map(PathBuf::from)
        .context("HOME is not set and CLI_MEMORY_DATA_DIR was not provided")?;
    Ok(home.join(".cli-memory-bridge-rs"))
}

fn resolved_model_dir_for(explicit_model_path: Option<PathBuf>, data_dir: &Path) -> PathBuf {
    explicit_model_path.unwrap_or_else(|| data_dir.join("models").join(DEFAULT_MODEL_DIRNAME))
}

fn ensure_model_cached_with<F>(
    model_dir: &Path,
    hf_cache_dir: &Path,
    token: Option<String>,
    mut fetcher: F,
) -> Result<()>
where
    F: FnMut(&str, &Path, Option<&str>) -> Result<PathBuf>,
{
    if required_model_files_present(model_dir) {
        return Ok(());
    }

    fs::create_dir_all(model_dir)
        .with_context(|| format!("failed to create model directory {}", model_dir.display()))?;
    fs::create_dir_all(hf_cache_dir)
        .with_context(|| format!("failed to create Hugging Face cache {}", hf_cache_dir.display()))?;

    for filename in REQUIRED_MODEL_FILES {
        let destination = model_dir.join(filename);
        if destination.is_file() {
            continue;
        }

        let source = fetcher(filename, hf_cache_dir, token.as_deref())
            .with_context(|| format!("failed to fetch {filename} for {DEFAULT_MODEL_REPO}"))?;
        copy_file_atomic(&source, &destination)?;
    }

    ensure!(
        required_model_files_present(model_dir),
        "model cache for {DEFAULT_MODEL_REPO} is incomplete at {}",
        model_dir.display()
    );
    Ok(())
}

fn required_model_files_present(model_dir: &Path) -> bool {
    REQUIRED_MODEL_FILES
        .iter()
        .all(|filename| model_dir.join(filename).is_file())
}

fn download_model_file(filename: &str, hf_cache_dir: &Path, token: Option<&str>) -> Result<PathBuf> {
    let api = ApiBuilder::from_cache(Cache::new(hf_cache_dir.to_path_buf()))
        .with_progress(false)
        .with_token(token.map(ToOwned::to_owned))
        .build()
        .context("failed to initialize Hugging Face API client")?;
    let repo = api.model(DEFAULT_MODEL_REPO.to_owned());
    let path = repo.get(filename)?;
    Ok(path)
}

fn copy_file_atomic(source: &Path, destination: &Path) -> Result<()> {
    let temp_path = destination.with_extension("download");
    fs::copy(source, &temp_path).with_context(|| {
        format!(
            "failed to copy downloaded model file from {} to {}",
            source.display(),
            temp_path.display()
        )
    })?;
    fs::rename(&temp_path, destination).with_context(|| {
        format!(
            "failed to move downloaded model file into place at {}",
            destination.display()
        )
    })?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{REQUIRED_MODEL_FILES, ensure_model_cached_with, resolved_model_dir_for};
    use std::{
        cell::Cell,
        fs,
        path::{Path, PathBuf},
    };

    #[test]
    fn explicit_model_path_overrides_default_cache_location() {
        let data_dir = PathBuf::from("/tmp/cli-memory-data");
        let explicit = PathBuf::from("/tmp/custom-model");
        let resolved = resolved_model_dir_for(Some(explicit.clone()), &data_dir);
        assert_eq!(resolved, explicit);
    }

    #[test]
    fn default_model_path_lives_under_app_models_directory() {
        let data_dir = PathBuf::from("/tmp/cli-memory-data");
        let resolved = resolved_model_dir_for(None, &data_dir);
        assert_eq!(
            resolved,
            PathBuf::from("/tmp/cli-memory-data/models/potion-multilingual-128M")
        );
    }

    #[test]
    fn cached_model_short_circuits_without_downloading() {
        let tempdir = tempfile::tempdir().expect("temporary directory should be created");
        let model_dir = tempdir.path().join("models/potion-multilingual-128M");
        let hf_cache_dir = tempdir.path().join(".hf-cache");
        fs::create_dir_all(&model_dir).expect("model directory should be created");
        for filename in REQUIRED_MODEL_FILES {
            fs::write(model_dir.join(filename), "cached").expect("cached file should be written");
        }

        let called = Cell::new(false);
        ensure_model_cached_with(&model_dir, &hf_cache_dir, None, |_file, _cache_dir, _token| {
            called.set(true);
            unreachable!("fetcher should not be called when cache is complete");
        })
        .expect("cache should validate");

        assert!(!called.get());
    }

    #[test]
    fn missing_model_files_are_downloaded_into_local_cache() {
        let tempdir = tempfile::tempdir().expect("temporary directory should be created");
        let model_dir = tempdir.path().join("models/potion-multilingual-128M");
        let hf_cache_dir = tempdir.path().join(".hf-cache");
        let source_dir = tempdir.path().join("download-source");
        fs::create_dir_all(&source_dir).expect("source directory should be created");

        ensure_model_cached_with(
            &model_dir,
            &hf_cache_dir,
            Some("token-123".to_owned()),
            |file, _cache_dir, token| fake_fetch(file, &source_dir, token),
        )
        .expect("model files should be fetched");

        for filename in REQUIRED_MODEL_FILES {
            let written = fs::read_to_string(model_dir.join(filename))
                .expect("downloaded model file should exist");
            assert_eq!(written, format!("downloaded:{filename}"));
        }
    }

    fn fake_fetch(file: &str, source_dir: &Path, token: Option<&str>) -> Result<PathBuf, anyhow::Error> {
        assert_eq!(token, Some("token-123"));
        let path = source_dir.join(file);
        fs::write(&path, format!("downloaded:{file}"))
            .expect("fake downloaded file should be written");
        Ok(path)
    }
}
