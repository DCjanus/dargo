use crate::DargoResult;
use colored::Colorize;
use std::{io::Write, path::PathBuf};
use structopt::StructOpt;
use toml_edit::Document;

#[derive(Debug, StructOpt)]
pub struct Rm {
    /// Dependencies to remove
    #[structopt(name = "dependencies", value_name = "dependency", required = true)]
    dependencies: Vec<String>,

    /// Path to the manifest to edit
    #[structopt(name = "manifest", long, value_name = "path", default_value = ".")]
    manifest: String,

    /// Remove build-dependencies
    #[structopt(name = "build", long)]
    build: bool,

    /// Remove dev-dependencies
    #[structopt(name = "dev", long, conflicts_with = "build")]
    dev: bool,

    /// Print changes to be made without actual make
    #[structopt(name = "dry", long)]
    dry: bool,
}

impl Rm {
    fn manifest_path(&self) -> DargoResult<PathBuf> {
        let mut manifest_path = PathBuf::from(&self.manifest);
        if manifest_path.is_dir() {
            manifest_path.push("Cargo.toml");
        }
        Ok(manifest_path)
    }

    fn dependencies_kind(&self) -> cargo::core::dependency::DepKind {
        if self.build {
            cargo::core::dependency::DepKind::Build
        } else if self.dev {
            cargo::core::dependency::DepKind::Development
        } else {
            cargo::core::dependency::DepKind::Normal
        }
    }

    pub fn run(self) -> DargoResult<()> {
        let manifest_path = &self.manifest_path()?;
        let manifest_text = std::fs::read_to_string(manifest_path)?;
        let mut document = manifest_text.parse::<Document>()?;
        let mut tw = tabwriter::TabWriter::new(vec![]);
        for dependency in &self.dependencies {
            let section = crate::crates::locate_dependency(self.dependencies_kind());
            if document[section][dependency].is_none() {
                warn!("did not find {} in {}", dependency, section);
                continue;
            }
            document[section][dependency] = toml_edit::Item::None;
            writeln!(tw, "Removing {}\t{}", dependency.bold(), section)?;
        }
        tw.flush()?;
        println!("{}", String::from_utf8(tw.into_inner()?)?);

        if !self.dry {
            std::fs::write(manifest_path, document.to_string())?;
        }

        Ok(())
    }
}
