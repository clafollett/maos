use clap::Parser;
use maos::cli::{Cli, CliContext};
use maos_core::ExitCode;

#[tokio::main(flavor = "current_thread")]
async fn main() -> std::process::ExitCode {
    // Parse command line arguments
    let cli = Cli::parse();

    // Build CLI context with lazy initialization for better startup performance
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
            tracing::error!("MAOS initialization failed: {err:?}");
            tracing::warn!("Check application logs for detailed error information");
            std::process::ExitCode::FAILURE
        }
    }
}
