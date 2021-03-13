pub use structopt::StructOpt;
use std::path;
// use quicli::prelude::*;
use crate::dir_scan;
// use crate::work_db;
// use crate::work_db::Dcm;

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

    match &args.action {
        Command::Find { path_to_dir_for_search } => {
            dir_scan::find_dcm_files(&path_to_dir_for_search);
        }
        Command::Depersonalize { path_to_dir_for_search, path_to_dir_for_save } => {
            println!("{:?} \n {:?}", path_to_dir_for_search, path_to_dir_for_save)
        }
    };
}