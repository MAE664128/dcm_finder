mod cli;
mod dir_scan;
mod work_dcm;
mod work_db;

use cli::start_cli;


fn main() -> Result<(), ()> {
    start_cli();

    Ok(())
}