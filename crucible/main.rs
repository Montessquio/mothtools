extern crate pest;
#[macro_use]
extern crate pest_derive;

use std::path::PathBuf;

use anyhow::{Result, bail};
use tracing::{event, Level};

mod parser;
use clap::{Parser, Subcommand, ValueEnum};
use tracing_subscriber::FmtSubscriber;

static LONG_ABOUT: &str = r#"Compile source code written in the Crucible programming language to Cultist Simulator JSON mod files.\n\"IN THE DESERT I WAIT IN THE RUINS I BURN - METAL IS WATER - STONE IS WAX - FLESH IS SMOKE - ENTER ME AND BE NO LONGER.\" - King Crucible"#;
/* 
#[derive(Parser, Debug)]
struct Args {
    #[arg(help = "The path to the directory containing your mod's `synopsis.json`. Your content files should be in `<MOD_ROOT>/src/content/`")]
    mod_root: PathBuf,

    #[arg(short, long, help = "The namespace your mod occupies. This will be prepended to the name of each output source file along with a dot.")]
    namespace: Option<String>,

    #[arg(short, long, action = clap::ArgAction::Count, conflicts_with = "quiet", help = "Increase log output. Use multiple times to further increase verbosity.")]
    verbose: u8,

    #[arg(short, long, action = clap::ArgAction::Count, conflicts_with = "verbose", help = "Reduce log output. Use multiple times to further decrease verbosity.")]
    quiet: u8,
}
*/

#[derive(Parser, Debug)]
#[command(author, version, about, about = "Compile source code written in the Crucible programming language to Cultist Simulator JSON mod files.", long_about = LONG_ABOUT)]
#[command(propagate_version = true)]
struct Args {
    /// A list of input files to operate on.
    /// If one or more directories are specified,
    /// Crucible will walk those directory trees
    /// and attempt to compile any file that ends
    /// with the `.crucible` extension.
    input: Vec<PathBuf>,

    /// If set to true, Crucible will compress its
    /// output and emit a `.lirc` file instead of `.lir`.
    #[arg(short, long, action = clap::ArgAction::SetTrue)]
    compress: bool,
    
    /// Specify a custom output file to emit to.
    /// If no path is specified, defaults to the
    /// standard output.
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Increase log output. Use multiple times to further increase verbosity.
    #[arg(global = true, short, long, action = clap::ArgAction::Count, conflicts_with = "quiet")]
    verbose: u8,

    /// Reduce log output. Use multiple times to further decrease verbosity.
    #[arg(global = true, short, long, action = clap::ArgAction::Count, conflicts_with = "verbose")]
    quiet: u8,
}

#[tokio::main]
async fn main() -> Result<()> {
    if let Err(e) = color_eyre::install() { bail!(e) };

    let cli = Args::parse();

    let level = if cli.verbose > 0 {
        match cli.verbose {
            0 => Level::INFO,
            1 => Level::DEBUG,
            2.. => Level::TRACE,
        }
    }
    else if cli.quiet > 0{
        match cli.quiet {
            0 | 3.. => Level::INFO,
            1 => Level::WARN,
            2 => Level::ERROR,
        }
    }
    else {
        Level::INFO
    };

    // Quiet > 2 means be totally silent - panics only.
    if cli.quiet <= 3 {
        let subscriber = FmtSubscriber::builder()
        .with_max_level(level)
        .finish();
        tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
    }

    let valid_paths = {
        let mut valid_paths: Vec<PathBuf> = Vec::new();
        for path in cli.input {
            let meta = std::fs::metadata(path.clone())
                .unwrap_or_else(|_| panic!("Path did not exist: {}", path.to_str().unwrap()));
            if meta.is_file() {
                valid_paths.push(path);
            }
            else {
                event!(
                    Level::ERROR, 
                    "Only file processing is currently supported. Path `{}` was type `{:?}`", 
                    path.to_str().unwrap(), 
                    std::fs::metadata(path.clone())
                        .unwrap_or_else(|_| panic!("Path did not exist: {}", path.to_str().unwrap()))
                        .file_type()
                );
                bail!("Invalid Operation")
            }
        }
        valid_paths
    };
    Ok(())
}