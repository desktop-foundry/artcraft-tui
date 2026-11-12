use tracing_subscriber::EnvFilter;

fn main() -> std::process::ExitCode {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();
    artcraft_cli::run()
}
