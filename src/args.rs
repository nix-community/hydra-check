use anyhow::bail;
use clap::{arg, builder::ArgPredicate, command, value_parser, CommandFactory, Parser};
use clap_complete::Shell;
use clap_verbosity_flag::{InfoLevel, Verbosity};
use flexi_logger::{Logger, LoggerHandle};
use log::{debug, error, warn};
use regex::Regex;
use std::{
    env::consts::{ARCH, OS},
    path::Path,
};

use crate::{constants, log_format, Evaluation, NixpkgsChannelVersion};

const DEFAULT_CHANNEL: &str = "unstable";

#[derive(Debug, Clone, Default)]
pub(crate) enum Queries {
    Jobset,
    Packages(Vec<String>),
    Evals(Vec<Evaluation>),

    /// No-op, e.g. when only printing shell completions
    #[default]
    Noop,
}

#[derive(Parser, Debug, Default)]
#[command(author, version, verbatim_doc_comment)]
#[allow(
    rustdoc::bare_urls,
    clippy::doc_markdown,
    clippy::struct_excessive_bools
)]
#[deny(missing_docs)]
///
/// Check hydra.nixos.org for build status of packages
///
/// Other channels can be:
///   - unstable      - alias for nixos/trunk-combined (for NixOS) or nixpkgs/trunk
///   - master        - alias for nixpkgs/trunk (Default for other architectures)
///   - staging-next  - alias for nixpkgs/staging-next
///   - 24.05         - alias for nixos/release-24.05
///
/// Usually using the above as --channel arguments, should fit most usages.
/// However, you can use a verbatim jobset name such as:
///
///   nixpkgs/nixpkgs-24.05-darwin
///
/// Jobset names can be constructed with the project name (e.g. `nixos/` or `nixpkgs/`)
/// followed by a branch name. The available jobsets can be found at:
///   - https://hydra.nixos.org/project/nixos
///   - https://hydra.nixos.org/project/nixpkgs
///
pub struct HydraCheckCli {
    #[arg(id = "PACKAGES")]
    queries: Vec<String>,

    /// Only print the hydra build url, then exit
    #[arg(long)]
    url: bool,

    /// Output json
    #[arg(long)]
    json: bool,

    /// Write only the latest build even if last build failed
    #[arg(short, long)]
    short: bool,

    /// Fetch more entries if possible (might be slower)
    #[arg(
        short,
        long,
        default_value_if("releases", ArgPredicate::IsPresent, "true")
    )]
    more: bool,

    /// System architecture to check
    #[arg(short, long)]
    arch: Option<String>,

    /// Channel to check packages for
    #[arg(short, long)]
    channel: Option<String>,

    /// Specify jobset to check packages for
    #[arg(long, conflicts_with = "channel")]
    jobset: Option<String>,

    /// Print details about specific evaluations instead of packages
    #[arg(short, long)]
    eval: bool,

    /// Query the release tests of the given channel (jobset)
    #[arg(
        short, long, conflicts_with_all = ["PACKAGES", "eval"],
        // --releases implies --tests
        default_value_if("releases", ArgPredicate::IsPresent, "true")
    )]
    tests: bool,

    /// Combine information from channel evals and release --tests
    #[arg(short, long, conflicts_with_all = ["PACKAGES", "eval"])]
    releases: bool,

    /// Print generated completions for a given shell
    #[arg(long = "shell-completion", exclusive = true, value_parser = value_parser!(Shell))]
    shell: Option<Shell>,

    #[command(flatten)]
    verbosity: Verbosity<InfoLevel>,
}

/// Resolved command line arguments, with all options normalized and unwrapped
#[derive(Debug, Default)]
#[allow(clippy::struct_excessive_bools)]
pub(crate) struct ResolvedArgs {
    /// List of packages or evals to query
    pub(crate) queries: Queries,
    pub(crate) url: bool,
    pub(crate) json: bool,
    pub(crate) short: bool,
    pub(crate) more: bool,
    pub(crate) releases: bool,
    pub(crate) channel: Option<String>,
    pub(crate) jobset: String,
}

impl HydraCheckCli {
    fn guess_arch(self) -> Self {
        let warn_if_unknown = |arch: &str| {
            if !Vec::from(constants::KNOWN_ARCHITECTURES).contains(&arch) {
                warn!(
                    "unknown --arch '{arch}', {}: {:#?}",
                    "consider specifying one of the following known architectures",
                    constants::KNOWN_ARCHITECTURES
                );
            }
        };
        if let Some(arch) = self.arch.clone() {
            // allow empty `--arch` as it may be the user's intention to
            // specify architectures explicitly for each package
            if !arch.is_empty() {
                warn_if_unknown(&arch);
            }
            return self;
        }
        let arch = format!(
            "{}-{}",
            ARCH,
            match OS {
                "macos" => "darwin", // hack to produce e.g. `aarch64-darwin`
                x => x,
            }
        );
        debug!("assuming --arch '{arch}'");
        warn_if_unknown(&arch);
        Self {
            arch: Some(arch),
            ..self
        }
    }

    /// Guesses the hydra jobset based on system information from build time,
    /// run time, and the provided command line arguments.
    /// Note that this method is inherently non-deterministic as it depends on
    /// the current build target & runtime systems.
    /// See the source code for the detailed heuristics.
    #[allow(clippy::missing_panics_doc)]
    pub fn guess_jobset(self) -> anyhow::Result<Self> {
        if self.jobset.is_some() {
            return Ok(Self {
                channel: None,
                ..self
            });
        }
        let channel = self.channel.unwrap_or(DEFAULT_CHANNEL.into());
        // https://wiki.nixos.org/wiki/Channel_branches
        // https://github.com/NixOS/infra/blob/master/channels.nix
        let (nixpkgs_unstable, nixos_unstable) = ("nixpkgs-unstable", "nixos-unstable");
        let channel_stable = |version: &str| {
            match self.arch {
                // darwin
                Some(ref x) if x.ends_with("darwin") => format!("nixpkgs-{version}-darwin"),
                // others
                _ => format!("nixos-{version}"),
            }
        };
        let channel: String = match channel.as_str() {
            "master" => nixpkgs_unstable.into(),
            DEFAULT_CHANNEL => match (Path::new("/etc/NIXOS").exists(), &self.arch) {
                (true, Some(arch))
                    if Vec::from(constants::NIXOS_ARCHITECTURES).contains(&arch.as_str()) =>
                {
                    // only returns the NixOS jobset if the current system is NixOS
                    // and the --arch is a NixOS supported system.
                    nixos_unstable.into()
                }
                _ => nixpkgs_unstable.into(),
            },
            "stable" => {
                let version = match NixpkgsChannelVersion::stable() {
                    Ok(version) => version,
                    Err(err) => {
                        error!(
                            "{}, {}.",
                            "could not fetch the stable release version number",
                            "please specify '--channel' or '--jobset' explicitly",
                        );
                        return Err(err);
                    }
                };
                channel_stable(&version)
            }
            x if Regex::new(r"^[0-9]+\.[0-9]+$").unwrap().is_match(x) => channel_stable(x),
            x => x.into(),
        };
        debug!("--channel resolves to '{channel}'");
        let jobset: String = match channel.as_str() {
            "nixpkgs-unstable" => "nixpkgs/trunk".into(),
            "nixos-unstable" => "nixos/trunk-combined".into(),
            "nixos-unstable-small" => "nixos/unstable-small".into(),
            // https://hydra.nixos.org/project/nixos
            x if x.starts_with("staging") && x.ends_with("-small") => format!("nixos/{x}"),
            // `nixos/staging` is abandoned while `nixpkgs/staging` is active
            // https://hydra.nixos.org/project/nixpkgs
            x if x.starts_with("staging") => format!("nixpkgs/{x}"),
            x if Regex::new(r"^nixos-[0-9]+\.[0-9]+").unwrap().is_match(x) => {
                x.replacen("nixos", "nixos/release", 1)
            }
            x if Regex::new(r"^nixpkgs-[0-9]+\.[0-9]+").unwrap().is_match(x) => {
                x.replacen("nixpkgs", "nixpkgs/nixpkgs", 1)
            }
            x => x.into(),
        };
        debug!("--channel '{channel}' implies --jobset '{jobset}'");
        Ok(Self {
            jobset: Some(jobset),
            channel: Some(channel),
            ..self
        })
    }

    /// Guesses the full package name spec (e.g. `nixpkgs.gimp.x86_64-linux`)
    /// for hydra, given the command line inputs.
    /// See the source code for the detailed heuristics.
    #[must_use]
    pub fn guess_package_name(&self, package: &str) -> String {
        let has_known_arch_suffix = constants::KNOWN_ARCHITECTURES
            .iter()
            .any(|known_arch| package.ends_with(format!(".{known_arch}").as_str()));

        let warn_unknown_arch = || -> String {
            warn!(
                "unknown architecture for package {package}, {}, {}, {}.",
                "consider specifying an arch suffix explicitly",
                "such as 'gimp.x86_64-linux'",
                "or provide a non-empty '--arch'"
            );
            "".into()
        };

        let arch_suffix = match self.arch.clone() {
            _ if has_known_arch_suffix => "".into(),
            None => warn_unknown_arch(),
            // empty --arch is useful for aggregate job such as the channel tests
            // e.g. https://hydra.nixos.org/job/nixpkgs/trunk/unstable
            Some(arch) if arch.is_empty() => "".into(),
            Some(arch) => format!(".{arch}"),
        };

        if package.starts_with("nixpkgs.") || package.starts_with("nixos.") {
            // we assume the user knows the full package name
            return format!("{package}{arch_suffix}");
        }

        if self.jobset.clone().is_some_and(|x| x.starts_with("nixos/")) {
            // we assume that the user searches for a package and not a test
            return format!("nixpkgs.{package}{arch_suffix}");
        }

        format!("{package}{arch_suffix}")
    }

    fn guess_packages(&self) -> anyhow::Result<Vec<String>> {
        if self.tests {
            let Some(ref jobset) = self.jobset else {
                bail!("--jobset is not properly set up or deduced");
            };
            // aggregate job for channel release tests; see the `job` keys in:
            // - https://github.com/NixOS/infra/blob/main/channels.nix, and
            // - https://status.nixos.org/
            //
            let aggregate_job = match jobset.as_str() {
                x if x.ends_with("darwin") => "darwin-tested",
                x if x.starts_with("nixpkgs/") => "unstable",
                x if x.starts_with("nixos/") => "tested",
                _ => {
                    let default = "tested";
                    warn!(
                        "unknown --jobset '{jobset}', assuming job '{default}' for release tests"
                    );
                    default
                }
            };
            return Ok(vec![aggregate_job.into()]);
        }
        let packages = self
            .queries
            .iter()
            .filter_map(|package| {
                if package.starts_with("python3Packages") || package.starts_with("python3.pkgs") {
                    error!(
                        "instead of '{package}', you want {}",
                        "python3xPackages... (e.g. python311Packages)"
                    );
                    None
                } else {
                    Some(self.guess_package_name(package))
                }
            })
            .collect();
        Ok(packages)
    }

    fn guess_evals(&self) -> Vec<Evaluation> {
        if self.queries.is_empty() {
            // this would resolve to the latest eval of a jobset:
            return vec![Evaluation::guess_from_spec("", self.more)];
        }
        let mut evals = Vec::new();
        for spec in &self.queries {
            evals.push(Evaluation::guess_from_spec(spec, self.more));
        }
        evals
    }

    /// Parses the command line flags, sets the log level, and calls
    /// [`Self::guess_all_args()`]. Also prints shell completions if asked for.
    pub(crate) fn parse_and_guess() -> anyhow::Result<(ResolvedArgs, LoggerHandle)> {
        let args = Self::parse();
        let log_handle = Logger::with(args.verbosity.log_level_filter())
            .format(log_format)
            .start()?;
        if let Some(shell) = args.shell {
            // generate shell completions
            let mut cmd = Self::command();
            let bin_name = cmd.get_name().to_string();
            let mut buf = Vec::new();
            clap_complete::generate(shell, &mut cmd, bin_name, &mut buf);
            let completion_text = String::from_utf8(buf)?;
            print!(
                "{}",
                match shell {
                    // hack to provide channel completions for zsh
                    Shell::Zsh => {
                        let channel_options = format!(
                            "CHANNEL:({})",
                            [
                                "nixpkgs-unstable",
                                "nixos-unstable",
                                "nixos-unstable-small",
                                "staging-next",
                                "stable"
                            ]
                            .join(" ")
                        );
                        let arch_options =
                            format!("ARCH:({})", constants::KNOWN_ARCHITECTURES.join(" "));
                        completion_text
                            .replace("CHANNEL:_default", &channel_options)
                            .replace("ARCH:_default", &arch_options)
                    }
                    _ => completion_text,
                }
            );
            let resolved_args = ResolvedArgs {
                queries: Queries::Noop,
                ..Default::default()
            };
            return Ok((resolved_args, log_handle));
        }
        Ok((args.guess_all_args()?, log_handle))
    }

    /// Guesses all relevant command line arguments.
    pub(crate) fn guess_all_args(self) -> anyhow::Result<ResolvedArgs> {
        let args = self;
        let args = args.guess_arch();
        let args = args.guess_jobset()?;
        let queries = match (args.eval, !args.queries.is_empty() || args.tests) {
            (true, _) => Queries::Evals(args.guess_evals()),
            (_, true) => Queries::Packages(args.guess_packages()?),
            (_, false) => Queries::Jobset,
        };
        Ok(ResolvedArgs {
            queries,
            url: args.url,
            json: args.json,
            short: args.short,
            more: args.more,
            releases: args.releases,
            channel: args.channel,
            jobset: args
                .jobset
                .expect("jobset should be resolved by `guess_jobset()`"),
        })
    }

    /// Runs the program and provides an exit code (with possible errors).
    pub fn execute() -> anyhow::Result<bool> {
        let (resolved_args, _logger_handle) = Self::parse_and_guess()?;
        resolved_args.fetch_and_print()
        // _logger_handle is dropped here
    }
}

impl ResolvedArgs {
    /// Fetches build or evaluation status from hydra.nixos.org
    /// and prints the result according to the command line specs.
    pub(crate) fn fetch_and_print(&self) -> anyhow::Result<bool> {
        match &self.queries {
            Queries::Jobset => {
                self.fetch_and_print_jobset(false)?;
                Ok(true)
            }
            Queries::Packages(packages) => self.fetch_and_print_packages(packages),
            Queries::Evals(evals) => self.fetch_and_print_evaluations(evals),
            Queries::Noop => Ok(true),
        }
    }
}

#[test]
fn guess_jobset() -> anyhow::Result<()> {
    let aliases = [
        ("24.05", "nixos/release-24.05"),
        ("nixos-23.05", "nixos/release-23.05"),
        ("nixos-23.11-small", "nixos/release-23.11-small"),
        ("nixpkgs-25.05-darwin", "nixpkgs/nixpkgs-25.05-darwin"),
        ("staging", "nixpkgs/staging"),
        ("staging-next", "nixpkgs/staging-next"),
        ("staging-next-small", "nixos/staging-next-small"),
        ("staging-next-24.11", "nixpkgs/staging-next-24.11"),
        ("staging-next-24.11-small", "nixos/staging-next-24.11-small"),
    ];
    for (channel, jobset) in aliases {
        eprintln!("{channel} => {jobset}");
        let args =
            HydraCheckCli::parse_from(["hydra-check", "--channel", channel]).guess_jobset()?;
        debug_assert_eq!(args.jobset, Some(jobset.into()));
    }
    Ok(())
}

#[test]
fn guess_darwin() -> anyhow::Result<()> {
    let apple_silicon = "aarch64-darwin";
    if Vec::from(constants::NIXOS_ARCHITECTURES).contains(&apple_silicon) {
        // if one day NixOS gains support for the darwin kernel
        // (however unlikely), abort this test
        return Ok(());
    }
    let args =
        HydraCheckCli::parse_from(["hydra-check", "--arch", apple_silicon]).guess_jobset()?;
    // always follow nixpkgs-unstable
    debug_assert_eq!(args.jobset, Some("nixpkgs/trunk".into()));
    Ok(())
}

#[test]
#[ignore = "require internet connection"]
fn guess_stable() -> anyhow::Result<()> {
    let args: HydraCheckCli =
        HydraCheckCli::parse_from(["hydra-check", "--channel", "stable"]).guess_jobset()?;
    eprintln!("{:?}", args.jobset);
    assert!(args.jobset.is_some_and(|x| x.starts_with("nixos/release-")));
    let args: HydraCheckCli = HydraCheckCli::parse_from([
        "hydra-check",
        "--channel",
        "stable",
        "--arch",
        "aarch64-darwin", // apple silicon
    ])
    .guess_jobset()?;
    eprintln!("{:?}", args.jobset);
    assert!(args
        .jobset
        .is_some_and(|x| x.starts_with("nixpkgs/nixpkgs-") && x.ends_with("darwin")));
    Ok(())
}
