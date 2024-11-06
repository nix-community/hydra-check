//! A module that formats the details of the specified (or inferred) packages,
//! e.g. from <https://hydra.nixos.org/job/nixpkgs/trunk/hello.x86_64-linux>.

use colored::Colorize;
use indexmap::IndexMap;
use log::warn;

use crate::{structs::BuildStatus, FetchHydra, ResolvedArgs, StatusIcon};

#[derive(Clone)]
/// Container for the build status and metadata of a package
struct PackageStatus<'a> {
    package: &'a str,
    url: String,
    /// Status of recent builds of the package
    builds: Vec<BuildStatus>,
}

impl FetchHydra for PackageStatus<'_> {
    fn get_url(&self) -> &str {
        &self.url
    }

    fn finish_with_error(self, status: String) -> Self {
        Self {
            builds: vec![BuildStatus {
                icon: StatusIcon::Warning,
                status,
                ..Default::default()
            }],
            ..self
        }
    }
}

impl<'a> PackageStatus<'a> {
    /// Initializes the status container with the resolved package name
    /// and the resolved command line arguments.
    fn from_package_with_args(package: &'a str, args: &'a ResolvedArgs) -> Self {
        //
        // Examples:
        // - https://hydra.nixos.org/job/nixos/release-19.09/nixpkgs.hello.x86_64-linux/latest
        // - https://hydra.nixos.org/job/nixos/release-19.09/nixos.tests.installer.simpleUefiGrub.aarch64-linux
        // - https://hydra.nixos.org/job/nixpkgs/trunk/hello.x86_64-linux/all
        //
        // There is also {url}/all which is a lot slower.
        //
        let url = format!("https://hydra.nixos.org/job/{}/{}", args.jobset, package);
        Self {
            package,
            url,
            builds: vec![],
        }
    }

    fn fetch_and_read(self) -> anyhow::Result<Self> {
        let doc = self.fetch_document()?;
        let tbody = match self.find_tbody(&doc, "") {
            Err(stat) => return Ok(stat),
            Ok(tbody) => tbody,
        };
        let builds = BuildStatus::from_tbody(tbody)?;
        Ok(Self { builds, ..self })
    }
}

impl ResolvedArgs {
    pub(crate) fn fetch_and_print_packages(&self, packages: &Vec<String>) -> anyhow::Result<bool> {
        let mut status = true;
        let mut indexmap = IndexMap::new();
        for (idx, package) in packages.iter().enumerate() {
            let stat = PackageStatus::from_package_with_args(package, self);
            if self.url {
                println!("{}", stat.get_url());
                continue;
            }
            let url_dimmed = stat.get_url().dimmed();
            if !self.json {
                // print title first, then fetch
                if idx > 0 && !self.short {
                    println!(""); // vertical whitespace
                }
                println!(
                    "Build Status for {} on jobset {}",
                    stat.package.bold(),
                    self.jobset.bold(),
                );
                if !self.short {
                    println!("{url_dimmed}");
                }
            }
            let stat = stat.fetch_and_read()?;
            let first_stat = stat.builds.first();
            let success = first_stat.is_some_and(|build| build.success);
            if !success {
                status = false;
            }
            if self.json {
                match self.short {
                    true => indexmap.insert(
                        stat.package,
                        match first_stat {
                            Some(x) => vec![x.to_owned()],
                            None => vec![],
                        },
                    ),
                    false => indexmap.insert(stat.package, stat.builds),
                };
                continue; // print later
            }
            println!("{}", stat.format_table(self.short, &stat.builds));
            if !success && self.short {
                warn!("latest build failed, check out: {}", url_dimmed)
            }
        }
        if self.json {
            println!("{}", serde_json::to_string_pretty(&indexmap)?);
        }
        Ok(status)
    }
}
