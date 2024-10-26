use hydra_check::{Args, PackageStatus, Queries, ResolvedArgs};
use std::borrow::Borrow;

fn query_packages(packages: &Vec<String>, args: &ResolvedArgs) -> anyhow::Result<bool> {
    let mut status = true;
    for (idx, package) in packages.iter().enumerate() {
        let pkg_stat = PackageStatus::from_package_with_args(package, &args);
        if idx > 0 {
            println!("");
        }
        let (success, output) = pkg_stat.fetch_and_format()?;
        if !success {
            status = false;
        }
        println!("{}", output);
    }
    Ok(status)
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse_and_guess()?;
    let success = match args.queries.borrow() {
        Queries::Jobset => todo!(),
        Queries::Packages(packages) => query_packages(packages, &args)?,
        Queries::Evals(evals) => todo!(),
    };
    if !success {
        std::process::exit(1);
    }
    Ok(())
}
