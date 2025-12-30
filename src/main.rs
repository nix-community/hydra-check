use hydra_check::HydraCheckCli;
use std::process::ExitCode;

fn main() -> anyhow::Result<ExitCode> {
    let success = HydraCheckCli::execute()?;
    if !success {
        return Ok(ExitCode::FAILURE);
    }

    Ok(ExitCode::SUCCESS)
}
