use hydra_check::HydraCheckCli;

fn main() -> anyhow::Result<()> {
    let success = HydraCheckCli::execute()?;
    if !success {
        std::process::exit(1);
    }
    Ok(())
}
