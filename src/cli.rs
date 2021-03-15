use std::path;
use std::time;
use crate::dir_scan;
pub use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "dcm finder", about = "Finder of study in dicom format, with the function of de-identification.")]
struct Cli {
    /// List of available commands in the utility:
    #[structopt(subcommand)]
    pub action: Command,

}

#[derive(Debug, StructOpt)]
enum Command {
    /// Search for DICOM files in directory
    Find {
        /// Input the path to the directory to search for DICOM files in it
        #[structopt(short = "p", long = "path", name = "find_in", parse(from_os_str))]
        path_to_dir_for_search: path::PathBuf,

    },
    /// Depersonalize all found DICOM files in the directory and save them in the specified directory.
    Depersonalize {
        /// Input the path to the directory to search for DICOM files in it
        #[structopt(short = "p", long = "path", name = "find_in", parse(from_os_str))]
        path_to_dir_for_search: path::PathBuf,

        /// Input the path to the directory where the de-identified DICOM files will be saved
        #[structopt(short = "s", long = "save", name = "save_in", parse(from_os_str))]
        path_to_dir_for_save: path::PathBuf,
    },
}


pub fn start_cli() {
    let args = Cli::from_args();
    let before = time::Instant::now();
    match &args.action {
        Command::Find { path_to_dir_for_search } => {
            dir_scan::scanning(&path_to_dir_for_search, true,None);
        }
        Command::Depersonalize { path_to_dir_for_search, path_to_dir_for_save } => {
            dir_scan::scanning(&path_to_dir_for_search, false, Some(&path_to_dir_for_save));
        }
    };
    println!("Elapsed time to complete: {:.2?}", before.elapsed());

}