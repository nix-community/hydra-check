//! A module that formats the details of the specified (or inferred) packages,
//! e.g. from <https://hydra.nixos.org/job/nixpkgs/trunk/hello.x86_64-linux>.

use colored::Colorize;
use indexmap::IndexMap;
use log::{debug, info};
use std::collections::VecDeque;

use super::builds::BuildReport;
use crate::{
    constants,
    queries::jobset::JobsetReport,
    structs::{BuildStatus, EvalStatus, ReleaseStatus},
    FetchHydraReport, ResolvedArgs, StatusIcon,
};

#[derive(Clone)]
/// Container for the build status and metadata of a package
struct PackageReport<'a> {
    package: &'a str,
    url: String,
    /// Status of recent builds of the package
    builds: Vec<BuildStatus>,
}

impl FetchHydraReport for PackageReport<'_> {
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

impl<'a> PackageReport<'a> {
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
        let url = format!(
            "{}/job/{}/{package}{}",
            &*constants::HYDRA_CHECK_HOST_URL,
            args.jobset,
            if args.more { "/all" } else { "" }
        );
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
    #[allow(clippy::too_many_lines)]
    pub(crate) fn fetch_and_print_packages(&self, packages: &[String]) -> anyhow::Result<bool> {
        let mut status = true;
        let mut all_builds = IndexMap::new();
        let mut all_releases = IndexMap::new();
        for (idx, package) in packages.iter().enumerate() {
            // postpone fetching until after the title is printed
            let stat = PackageReport::from_package_with_args(package, self);
            if self.url {
                println!("{}", stat.get_url());
                continue;
            }
            let url_dimmed = stat.get_url().dimmed();
            let maybe_channel = self.channel.as_deref();
            let (channel, jobset_report) = if self.releases {
                let channel = maybe_channel.expect("--channel must be set when --releases is used");
                info!(
                    "fetching recent evals on --channel {} for --releases",
                    channel.bold()
                );
                let jobset_report = JobsetReport::from(self).fetch_and_read()?;
                eprintln!();
                (channel, Some(jobset_report))
            } else {
                (maybe_channel.unwrap_or_default(), None)
            };
            if !self.json {
                // print title first, then fetch
                if idx > 0 && !self.short {
                    println!(); // vertical whitespace
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
            let release_stats = if let Some(jobset_report) = jobset_report {
                // mutable refs that is quick to remove from the front
                let mut test_builds: VecDeque<&BuildStatus> = stat.builds.iter().collect();

                // this captures `test_builds` mutably but it does _not_ need
                // to be marked as `mut` because it is moved into .filter_map()
                // and re-borrowed as mut by them.
                let filter_eval = |eval: EvalStatus| {
                    let short_rev = eval.short_rev.as_deref().unwrap_or_default();
                    for index in 0..test_builds.len() {
                        #[allow(clippy::redundant_else)]
                        if test_builds[index]
                            .name
                            .as_deref()
                            .unwrap_or_default()
                            .contains(short_rev)
                        {
                            let test = test_builds.remove(index)?.clone();
                            return Some(ReleaseStatus::new(eval, test, channel));
                        } else {
                            debug!("skipping build: {:?}", test_builds[index]);
                        }
                    }
                    None
                };
                jobset_report
                    .evals
                    .into_iter()
                    .filter_map(filter_eval)
                    .collect()
            } else {
                vec![]
            };
            if self.json {
                if self.releases {
                    let release_stats = match self.short {
                        true => release_stats.first().cloned().into_iter().collect(),
                        false => release_stats,
                    };
                    all_releases.insert(channel, release_stats);
                } else {
                    let build_stats = match self.short {
                        true => first_stat.cloned().into_iter().collect(),
                        false => stat.builds,
                    };
                    all_builds.insert(stat.package, build_stats);
                }
                continue; // print later
            }
            match self.releases {
                true => println!("{}", stat.format_table(self.short, &release_stats)),
                false => println!("{}", stat.format_table(self.short, &stat.builds)),
            }
            let url_stripped = stat.get_url().trim_end_matches("/all");
            if !success {
                if self.short {
                    info!("latest build failed, check out: {url_dimmed}");
                } else {
                    eprintln!("\n{}", "Links:".bold());
                    #[rustfmt::skip]
                    eprintln!(
                        "{} (all builds)",
                        format!("🔗 {url_stripped}/all").dimmed()
                    );
                    eprintln!(
                        "{} (latest successful build)",
                        format!("🔗 {url_stripped}/latest").dimmed()
                    );
                    eprintln!(
                        "{} (latest success from a finished eval)",
                        format!("🔗 {url_stripped}/latest-finished").dimmed()
                    );

                    eprintln!();
                }
                info!("showing inputs for the latest success from a finished eval...");

                let url = format!("{url_stripped}/latest-finished");
                let build_report = BuildReport::from_url(&url).fetch_and_read()?;
                for entry in &build_report.inputs {
                    if self.short {
                        if let (Some(name), Some(rev)) = (&entry.name, &entry.revision) {
                            println!("{name}: {rev}");
                        }
                    } else {
                        println!(); // vertical separation
                        println!("{entry}");
                    }
                }
            }
        }
        if self.json {
            match self.releases {
                true => println!("{}", serde_json::to_string_pretty(&all_releases)?),
                false => println!("{}", serde_json::to_string_pretty(&all_builds)?),
            }
        }
        Ok(status)
    }
}
