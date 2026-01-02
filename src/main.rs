use clap::Parser;
use std::path::PathBuf;

mod converter;

#[derive(Parser)]
#[command(name = "XML Overview")]
#[command(version)]
#[command(about = "Generates an overview of an XML file.")]
struct Args {
    /// The XML file to be converted.
    #[arg(short, long)]
    file: PathBuf,

    /// (Optional) The maximum depth of the XML tree that should be considered. 
    /// Leave empty to read the whole XML structure.
    #[arg(short, long)]
    depth: Option<usize>,

    /// (Optional) The path of the output file where the XML overview should be written, leave empty to print to stdout.
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// (Optional) Show extra comments in the overview that give extra information related to the original XML, like how many XML tags were omitted in a certain position.
    #[arg(short, long, default_value_t=false)]
    verbose: bool,
}

fn main() {
    let args = Args::parse();

    converter::convert(&args.file);
}