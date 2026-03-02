use trekr::cli;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let command = cli::parse_app_command().map_err(|err| format!("argument error: {err}"))?;
    cli::execute_app_command(command)
}
