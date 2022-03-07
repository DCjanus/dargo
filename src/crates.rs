use crate::DargoResult;
use cargo::core::dependency::DepKind;
use cargo::util::VersionExt;
use cargo::{
    core::{source::Source, Dependency, SourceId, Summary},
    sources::SourceConfigMap,
};
use cargo_platform::Platform;
use semver::{Version, VersionReq};
use toml_edit::Document;

pub fn update_index(source_id: SourceId) -> DargoResult<()> {
    // XXX better look for updating index
    SourceConfigMap::new(&cargo::Config::default()?)?
        .load(source_id, &Default::default())?
        .update()?;
    Ok(())
}

pub fn latest_version_fuzzy(
    name: &str,
    source_id: SourceId,
    version_req: VersionReq,
    allow_prerelease: bool,
) -> DargoResult<Option<(String, Version)>> {
    if let Some(version) = latest_version(name, source_id, version_req.clone(), allow_prerelease)? {
        return Ok(Some((name.to_string(), version)));
    }

    let mut name = name.to_string();
    let positions: Vec<usize> = name
        .bytes()
        .enumerate()
        .filter(|(_, item)| *item == b'-' || *item == b'_')
        .map(|(index, _)| index)
        .collect::<Vec<_>>();
    match positions.len() {
        0 => return Ok(None),
        1..=127 => {}
        _ => return Err(format_err!("crate name contain too many '-' or '_'")),
    }

    for mask in 0..u128::pow(2, positions.len() as u32) {
        positions.iter().enumerate().for_each(|(index, item)| {
            #[allow(unsafe_code)]
            unsafe {
                name.as_bytes_mut()[*item] = match (mask >> index) & 1 {
                    0 => b'_',
                    1 => b'-',
                    _ => unreachable!(),
                }
            };
        });
        if let Some(version) =
            latest_version(&name, source_id, version_req.clone(), allow_prerelease)?
        {
            return Ok(Some((name, version)));
        }
    }

    Ok(None)
}

pub fn latest_version(
    name: &str,
    source_id: SourceId,
    version_req: VersionReq,
    allow_prerelease: bool,
) -> DargoResult<Option<Version>> {
    let config = cargo::Config::default()?;
    let source_config_map = SourceConfigMap::new(&config)?;
    let mut dependency = Dependency::parse(name, None, source_id)?;
    dependency.set_version_req(version_req);

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

pub fn locate_dependency(kind: DepKind) -> &'static str {
    match kind {
        DepKind::Normal => "dependencies",
        DepKind::Development => "dev-dependencies",
        DepKind::Build => "build-dependencies",
    }
}

pub fn get_dependency_version_req_text<'a>(
    document: &'a Document,
    kind: DepKind,
    platform: Option<&Platform>,
    name_in_toml: &str,
) -> Option<&'a str> {
    let item = match platform {
        None => &document[locate_dependency(kind)][name_in_toml],
        Some(platform) => {
            &document["target"][platform.to_string()][locate_dependency(kind)][name_in_toml]
        }
    };

    if item.is_str() {
        item.as_str()
    } else {
        item["version"].as_str()
    }
}

pub fn put_dependency_version_req_text(
    document: &mut Document,
    kind: DepKind,
    platform: Option<&Platform>,
    name_in_toml: &str,
    new_text: &str,
) {
    let item = match platform {
        None => &mut document[locate_dependency(kind)].or_insert(toml_edit::table())[name_in_toml],
        Some(platform) => &mut document["target"][platform.to_string()][locate_dependency(kind)]
            .or_insert(toml_edit::table())[name_in_toml],
    };

    let new_value = toml_edit::value(new_text);
    if item.is_str() || item.is_none() {
        *item = new_value;
    } else {
        item["version"] = new_value;
    }
}
