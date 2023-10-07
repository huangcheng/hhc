use clap::Parser;
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub(crate) struct Args {
    /// The path of files to be converted. If not specified, read from current directory.
    #[arg(short, long)]
    pub path: Option<String>,

    /// If original files should be overwritten by converted files. Default is false.
    #[arg(short, long, default_value_t = false)]
    pub overwrite: bool,
}
