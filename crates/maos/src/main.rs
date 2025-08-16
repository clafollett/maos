use clap::Parser;
use maos::cli::{Cli, CliContext};
use maos_core::ExitCode;

#[tokio::main]
async fn main() -> std::process::ExitCode {
    // Parse command line arguments
    let cli = Cli::parse();

    // Build the CLI context with dispatcher and handlers
    match CliContext::build().await {
        Ok(context) => {
            // Execute the command through the dispatcher
            let exit_code = context.execute(cli.command).await;

            // Convert MAOS exit code to process exit code
            match exit_code {
                ExitCode::Success => std::process::ExitCode::SUCCESS,
                _ => std::process::ExitCode::from(exit_code as u8),
            }
        }
        Err(err) => {
            // ✅ STDOUT CONTROL REMOVED: Use structured logging instead of eprintln!
            tracing::error!("MAOS initialization failed: {err:?}");
            tracing::warn!("Check application logs for detailed error information");
            std::process::ExitCode::FAILURE
        }
    }
}
