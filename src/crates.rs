use crate::DargoResult;
use cargo::{
    core::{
        dependency::{Kind as DependencyKind, Platform},
        source::Source,
        Dependency, SourceId, Summary,
    },
    sources::SourceConfigMap,
};
use semver::Version;
use toml_edit::Document;

pub fn update_index(source_id: SourceId) -> DargoResult<()> {
    // XXX better look for updating index
    SourceConfigMap::new(&cargo::Config::default()?)?
        .load(source_id, &Default::default())?
        .update()?;
    Ok(())
}

pub fn latest_version(
    name: &str,
    source_id: SourceId,
    allow_prerelease: bool,
) -> DargoResult<Option<Version>> {
    let config = cargo::Config::default()?;
    let source_config_map = SourceConfigMap::new(&config)?;
    let dependency = Dependency::parse_no_deprecated(name, None, source_id)?;

    let mut result: Option<Version> = None;

    source_config_map
        .load(dependency.source_id(), &Default::default())?
        .query(&dependency, &mut |summary: Summary| {
            if summary.version().is_prerelease() && !allow_prerelease {
                return;
            }

            if result.is_none() || result.as_ref().unwrap() < summary.version() {
                result.replace(summary.version().clone());
            }
        })?;

    Ok(result)
}

pub fn locate_dependency(kind: DependencyKind) -> &'static str {
    match kind {
        DependencyKind::Normal => "dependencies",
        DependencyKind::Development => "dev-dependencies",
        DependencyKind::Build => "build-dependencies",
    }
}

pub fn get_dependency_version_req_text<'a>(
    document: &'a Document,
    kind: DependencyKind,
    platform: Option<&Platform>,
    name_in_toml: &str,
) -> &'a str {
    let item = match platform {
        None => &document[locate_dependency(kind)][name_in_toml],
        Some(platform) => {
            &document["target"][platform.to_string()][locate_dependency(kind)][name_in_toml]
        }
    };

    if item.is_str() {
        item.as_str().unwrap()
    } else {
        item["version"].as_str().unwrap()
    }
}

pub fn put_dependency_version_req_text(
    document: &mut Document,
    kind: DependencyKind,
    platform: Option<&Platform>,
    name_in_toml: &str,
    new_text: &str,
) {
    let item = match platform {
        None => &mut document[locate_dependency(kind)][name_in_toml],
        Some(platform) => {
            &mut document["target"][platform.to_string()][locate_dependency(kind)][name_in_toml]
        }
    };

    let new_value = toml_edit::value(new_text);
    if item.is_str() {
        *item = new_value;
    } else {
        item["version"] = new_value;
    }
}
