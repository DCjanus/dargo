use crate::DargoResult;
use cargo::core::{SourceId, Workspace};
use colored::Colorize;
use semver::VersionReq;
use std::{io::Write, path::PathBuf};
use structopt::StructOpt;
use toml_edit::Document;

#[derive(Debug, StructOpt)]
pub struct Add {
    /// Dependencies to add, only crate name or '{name}@{version}' or '{name}@{version_requirement}', e.g. 'futures-preview@0.3.0-alpha.16', 'libc@>=0.1,<1.0'
    #[structopt(name = "dependencies", value_name = "dependency", required = true)]
    dependencies: Vec<String>,

    /// Path to the manifest to edit
    #[structopt(name = "manifest", long, value_name = "path", default_value = ".")]
    manifest: String,

    /// Add dev-dependencies
    #[structopt(name = "dev", long, conflicts_with = "build")]
    dev: bool,

    /// Add build-dependencies
    #[structopt(name = "build", long)]
    build: bool,

    /// Include prerelease versions when try to add(e,g. "0.3.0-alpha.15")
    #[structopt(name = "pre", long)]
    pre: bool,

    /// Print changes to be made without actual make
    #[structopt(name = "dry", long)]
    dry: bool,

    /// Update index before query latest version
    #[structopt(name = "update", long)]
    update: bool,
}

impl Add {
    fn manifest_path(&self) -> DargoResult<PathBuf> {
        let mut manifest_path = PathBuf::from(&self.manifest);
        if manifest_path.is_dir() {
            manifest_path.push("Cargo.toml");
        }
        Ok(manifest_path.canonicalize()?)
    }

    fn kind(&self) -> cargo::core::dependency::Kind {
        if self.dev {
            cargo::core::dependency::Kind::Development
        } else if self.build {
            cargo::core::dependency::Kind::Build
        } else {
            cargo::core::dependency::Kind::Normal
        }
    }

    pub fn run(self) -> DargoResult<()> {
        let manifest_path = self.manifest_path()?;
        let config = &cargo::Config::default()?;
        let source_id = SourceId::crates_io(&config)?;
        let workspace = Workspace::new(&manifest_path, config)?;
        if workspace.is_virtual() {
            return Err(format_err!("This is a virtual workspace"));
        }
        if self.update {
            crate::crates::update_index(source_id)?;
        }

        let mut document = std::fs::read_to_string(&manifest_path)?.parse::<Document>()?;

        let mut tw = tabwriter::TabWriter::new(vec![]);
        for crate_name in &self.dependencies {
            let (name, version_req) = match crate_name.splitn(2, '@').collect::<Vec<_>>().as_slice()
            {
                [name] => (name.to_string(), None),
                [name, version_req] => (name.to_string(), Some(VersionReq::parse(version_req)?)),
                _ => unreachable!(),
            };

            let (actual_name, latest_version) = match crate::crates::latest_version_fuzzy(
                &name,
                source_id,
                version_req.clone().unwrap_or_else(VersionReq::any),
                self.pre,
            )? {
                None => {
                    warn!("no available versions found for {}", name);
                    continue;
                }
                Some(x) => {
                    if !x.0.eq(&name) {
                        warn!("Added `{}` instead of `{}`", x.0, name);
                    }
                    x
                }
            };

            let name = actual_name;
            let version = match version_req {
                None => latest_version.to_string(),
                Some(x) => x.to_string(),
            };

            if crate::crates::get_dependency_version_req_text(&document, self.kind(), None, &name)
                .is_some()
            {
                warn!(
                    "{} already exists in {}",
                    name,
                    crate::crates::locate_dependency(self.kind())
                );
                continue;
            }

            writeln!(
                tw,
                "Adding {}\t{}\t{}",
                name,
                version.as_str().bright_green(),
                crate::crates::locate_dependency(self.kind())
            )?;

            crate::crates::put_dependency_version_req_text(
                &mut document,
                self.kind(),
                None,
                &name,
                &version,
            );
        }

        tw.flush()?;
        println!("{}", String::from_utf8(tw.into_inner()?)?);

        if !self.dry {
            std::fs::write(manifest_path, document.to_string())?;
        }

        Ok(())
    }
}
