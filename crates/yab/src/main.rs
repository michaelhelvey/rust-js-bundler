use color_eyre::Result;
use resolve::get_import_statements;
use tokio::fs::File;
use tracing::debug;

mod cli;
mod resolve;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    tracing_subscriber::fmt::init();

    let args = cli::parse_config();
    let file = File::open(&args.entrypoint).await?;

    let imports = get_import_statements(file, &args.entrypoint).await?;
    debug!("imports for file {:?}, {:#?}", args.entrypoint, imports);

    for import in imports {
        let resolved = resolve::resolve_import(&import.specifier, &import.relative_to).await?;
        debug!("resolved import {:?} to {:?}", import.specifier, resolved);
    }

    Ok(())
}
