use trekr::cli;
use trekr::diagnostics;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    diagnostics::install_panic_logging();
    diagnostics::log_info("startup", "trekr process started");
    let command = cli::parse_app_command().map_err(|err| format!("argument error: {err}"))?;
    let result = cli::execute_app_command(command);
    if let Err(error) = &result {
        diagnostics::log_error("startup", format!("process exiting with error: {error}"));
    } else {
        diagnostics::log_info("startup", "trekr process exited successfully");
    }
    result
}
