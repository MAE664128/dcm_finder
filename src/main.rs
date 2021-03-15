mod cli;
mod dir_scan;
mod work_dcm;
mod work_db;

use cli as dcm_finder_cli;


fn main() -> Result<(), ()> {
    dcm_finder_cli::start_cli();
    Ok(())
}