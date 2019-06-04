use crate::DargoResult;
use cargo::core::{Dependency, SourceId, Workspace};
use colored::Colorize;
use semver::{Version, VersionReq};
use std::{collections::HashSet, io::Write, path::PathBuf};
use structopt::StructOpt;
use toml_edit::Document;

#[derive(Debug, StructOpt)]
pub struct Upgrade {
    /// Path to the manifest to upgrade
    #[structopt(name = "manifest", long, value_name = "path", default_value = ".")]
    manifest: String,

    /// Upgrade dependencies only these
    #[structopt(
        name = "only",
        long,
        value_name = "dependencies",
        conflicts_with = "exclude"
    )]
    only: Vec<String>,

    /// Upgrade dependencies exclude these
    #[structopt(name = "exclude", long, value_name = "dependencies")]
    exclude: Vec<String>,

    /// Include prerelease versions when try to upgrade(e,g. "0.3.0-alpha.15")
    #[structopt(name = "pre", long)]
    pre: bool,

    /// Print changes to be made without actual make
    #[structopt(name = "dry", long)]
    dry: bool,

    /// Update index before query latest version
    #[structopt(name = "update", long)]
    update: bool,

    /// Upgrade all kinds of version requirements to latest
    #[structopt(name = "force", long)]
    force: bool,
}

#[derive(Debug)]
struct UpgradeTask {
    dependency: Dependency,
    pre_version: String,
    new_version: String,
}

impl Upgrade {
    fn manifest_path(&self) -> DargoResult<PathBuf> {
        let mut manifest_path = PathBuf::from(&self.manifest);
        if manifest_path.is_dir() {
            manifest_path.push("Cargo.toml");
        }
        Ok(manifest_path.canonicalize()?)
    }

    fn gen_upgrade_task(
        &self,
        document: &Document,
        dependency: &Dependency,
    ) -> DargoResult<Option<UpgradeTask>> {
        if !dependency.source_id().is_registry() {
            // TODO support local registry in the future
            return Ok(None);
        }

        let version_req_text = crate::crates::get_dependency_version_req_text(
            document,
            dependency.kind(),
            dependency.platform(),
            dependency.name_in_toml().as_str(),
        )
        .unwrap();

        let latest_version = match crate::crates::latest_version(
            dependency.package_name().as_str(),
            dependency.source_id(),
            VersionReq::any(),
            self.pre,
        )? {
            None => {
                warn!(
                    "no available versions found for {}",
                    dependency.package_name()
                );
                return Ok(None);
            }
            Some(x) => x,
        };

        let current_version = match Version::parse(version_req_text) {
            Ok(x) => x, // Version requirements like '0.1.0', rather than '^0.1.0', '0.1', '~0.1.0', '1.*', '>=0.1.0'
            Err(_) => {
                if self.force {
                    // Upgrade all version requirements
                    let task = UpgradeTask {
                        dependency: dependency.clone(),
                        pre_version: version_req_text.to_string(),
                        new_version: latest_version.to_string(),
                    };
                    return Ok(Some(task));
                } else {
                    // Ignore version requirements like '^0.1.0', '0.1', '~0.1.0', '1.*', '>=0.1.0'
                    return Ok(None);
                }
            }
        };

        if current_version == latest_version {
            Ok(None)
        } else {
            let task = UpgradeTask {
                dependency: dependency.clone(),
                pre_version: version_req_text.to_string(),
                new_version: latest_version.to_string(),
            };
            Ok(Some(task))
        }
    }

    pub fn run(self) -> DargoResult<()> {
        let config = &cargo::Config::default()?;
        let workspace = Workspace::new(&self.manifest_path()?, config)?;

        let mut index_updated: HashSet<SourceId> = HashSet::new();
        let only = self
            .only
            .iter()
            .map(String::as_str)
            .collect::<HashSet<&str>>();
        let exclude = self
            .exclude
            .iter()
            .map(String::as_str)
            .collect::<HashSet<&str>>();

        for package in workspace.members() {
            let manifest_path = package.manifest_path();
            let manifest_text = std::fs::read_to_string(manifest_path)?;
            let mut document: Document = manifest_text.parse()?;

            let mut upgrade_tasks = Vec::new();

            for dependency in package.dependencies() {
                if !only.is_empty() && !only.contains(dependency.name_in_toml().as_str()) {
                    continue;
                }
                if !exclude.is_empty() && exclude.contains(dependency.name_in_toml().as_str()) {
                    continue;
                }
                if self.update && index_updated.insert(dependency.source_id()) {
                    crate::crates::update_index(dependency.source_id())?;
                }
                if let Some(task) = self.gen_upgrade_task(&document, dependency)? {
                    upgrade_tasks.push(task);
                }
            }

            let mut tw = tabwriter::TabWriter::new(vec![]);
            writeln!(tw, "{}:", package.name().magenta())?;
            for task in &upgrade_tasks {
                writeln!(
                    tw,
                    "{}\t{}\t{}",
                    task.dependency.name_in_toml(),
                    task.pre_version.strikethrough(),
                    task.new_version.bright_green(),
                )?;
                crate::crates::put_dependency_version_req_text(
                    &mut document,
                    task.dependency.kind(),
                    task.dependency.platform(),
                    &task.dependency.name_in_toml(),
                    &task.new_version,
                );
            }
            tw.flush()?;
            println!("{}", String::from_utf8(tw.into_inner()?)?);

            if !self.dry {
                std::fs::write(manifest_path, document.to_string())?;
            }
        }

        Ok(())
    }
}
