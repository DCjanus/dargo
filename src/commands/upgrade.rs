use crate::DargoResult;
use cargo::core::{Dependency, Workspace};
use colored::Colorize;
use semver::Version;
use std::{env::current_dir, io::Write, path::PathBuf};
use structopt::StructOpt;
use toml_edit::Document;

#[derive(Debug, StructOpt)]
pub struct Upgrade {
    /// Path to the manifest to upgrade
    #[structopt(long, value_name = "path")]
    manifest: Option<String>,

    /// Include prerelease versions when fetching from crates.io(e,g. "0.3.0-alpha.15")
    #[structopt(long)]
    pre: bool,

    /// Print changes to be made without actual make.
    #[structopt(long, short)]
    dry: bool,

    /// TODO: Update index before query latest version
    #[structopt(long, short)]
    update: bool,
}

#[derive(Debug)]
struct UpgradeTask {
    dependency: Dependency,
    pre_version: String,
    new_version: String,
}

impl Upgrade {
    fn manifest(&self) -> DargoResult<PathBuf> {
        let value = match &self.manifest {
            None => return Ok(current_dir()?.join("Cargo.toml")),
            Some(x) => x,
        };

        let abspath = PathBuf::from(value).canonicalize()?;
        if abspath.is_dir() {
            Ok(abspath.join("Cargo.toml"))
        } else {
            Ok(abspath)
        }
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
        );

        let latest_version = match crate::crates::latest_version(
            dependency.package_name().as_str(),
            dependency.source_id(),
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

        let result = if let Ok(current_version) = Version::parse(version_req_text) {
            // Version requirements like: 0.1.2, 2.1.0, 3.1.12-beta
            if current_version == latest_version {
                None
            } else {
                Some(latest_version)
            }
        } else {
            // Version requirements like: =0.1.2, <1.2.3, >= 2.3.0 ...
            if dependency.version_req().matches(&latest_version) {
                None
            } else {
                Some(latest_version)
            }
        }
        .map(|version: Version| UpgradeTask {
            dependency: dependency.clone(),
            pre_version: version_req_text.to_string(),
            new_version: version.to_string(),
        });
        Ok(result)
    }

    pub fn run(self) -> DargoResult<()> {
        let config = &cargo::Config::default()?;
        let workspace = Workspace::new(&self.manifest()?, config)?;

        for package in workspace.members() {
            let manifest_path = package.manifest_path();
            let manifest_text = file::get_text(manifest_path)?;
            let mut document: Document = manifest_text.parse()?;

            let upgrade_tasks: Vec<UpgradeTask> = package
                .dependencies()
                .iter()
                .map(|dependency: &Dependency| self.gen_upgrade_task(&document, dependency))
                .filter_map(|x: DargoResult<Option<UpgradeTask>>| match x {
                    Ok(Some(task)) => Some(Ok(task)),
                    Ok(None) => None,
                    Err(error) => Some(Err(error)),
                })
                .collect::<DargoResult<Vec<UpgradeTask>>>()?;

            println!("{}:", package.name().magenta());
            let mut tw = tabwriter::TabWriter::new(vec![]);
            for task in &upgrade_tasks {
                writeln!(
                    tw,
                    "{}\t{} -> {}",
                    task.dependency.name_in_toml(),
                    task.pre_version.strikethrough(),
                    task.new_version.bright_green(),
                )?;
            }
            tw.flush()?;
            println!("{}", String::from_utf8(tw.into_inner()?)?);

            if !self.dry {
                for task in upgrade_tasks {
                    crate::crates::put_dependency_version_req_text(
                        &mut document,
                        task.dependency.kind(),
                        task.dependency.platform(),
                        &task.dependency.name_in_toml(),
                        &task.new_version,
                    );
                }
                file::put_text(manifest_path, document.to_string())?;
            }
        }

        Ok(())
    }
}
