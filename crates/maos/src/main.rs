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
        Err(_err) => {
            // 🔒 SECURITY FIX: Don't leak error details to stdout/stderr
            eprintln!("Error initializing MAOS - check logs for details");
            std::process::ExitCode::FAILURE
        }
    }
}
