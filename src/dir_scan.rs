extern crate indicatif;
use std::fs;
use std::path;
use std::convert::TryInto;
use std::sync::{Arc, Mutex};
use walkdir::{DirEntry, WalkDir};
use indicatif::ParallelProgressIterator;
use rayon::iter::{ParallelIterator, IntoParallelRefIterator};
use crate::work_db;
use crate::work_db::Dcm;

use crate::work_dcm;
// use std::ffi::OsStr;
// use rand;
// use rand::Rng;
// use rand::distributions;
// use std::io::Error;
// use std::fs::ReadDir;
// use rayon::join_context;

/// Проверяет, является ли директория скрытой
fn is_hidden(entry: &DirEntry) -> bool {
    entry.file_name()
        .to_str()
        .map(|s| s.starts_with("."))
        .unwrap_or(false)
}

/// Выполняет рекурсивный поиск всех файлов в директории
/// Поиск не выполняется в скрытых директориях
/// Возвращает вектор путей
fn find_all_files(dir_path: &path::PathBuf) -> Vec<path::PathBuf> {
    WalkDir::new(dir_path)
        .into_iter()
        .filter_entry(|e| !is_hidden(e))
        .filter_map(|e| e.ok())
        .map(|file| file.path().to_owned())
        .collect()
}

/// Выполняет рекурсивный поиск всех DICOM файлов в директории с
/// параллельной итерацией при выполнении операции чтения
/// Поиск не выполняется в скрытых директориях
pub fn find_dcm_files(dir_path: &path::PathBuf) {
    let paths = find_all_files(dir_path);
    println!("Total files found: {}", paths.len());

    match work_db::Connection::create_dcm_tables(true) {
        Ok(conn) => {
            let contents = Arc::new(Mutex::new(conn));
            paths.par_iter()
                .progress_count(paths.len().try_into().unwrap_or_default())
                .for_each(|path| {
                    match work_dcm::read_dcm(&path.as_path()) {
                        Ok(dcm_obj) => {
                            let meta_dcm = work_dcm::MetaDcm::from(
                                &dcm_obj,
                                &path.as_path().to_str().unwrap_or_default(),
                            );
                            add_dcm_in_contents(&contents, &meta_dcm);
                        }
                        Err(_) => {}
                    };
                });
            export_res_as_json(contents);
        }
        Err(error) => {
            eprintln!("Error create data base in memory: {:?}", error)
        }
    }
}

/// Выполняет рекурсивный поиск всех DICOM файлов в директории с
/// параллельной итерацией при выполнении операции чтения
/// Поиск не выполняется в скрытых директориях
pub fn de_identification_dcm_files(find_in: &path::PathBuf, save_in: &path::PathBuf) {
    let paths = find_all_files(find_in);
    println!("Total files found: {}", paths.len());

    match work_db::Connection::create_dcm_tables(true) {
        Ok(conn) => {
            let contents = Arc::new(Mutex::new(conn));
            paths.par_iter()
                .progress_count(paths.len().try_into().unwrap_or_default())
                .for_each(|path| {
                    match work_dcm::read_dcm(&path.as_path()) {
                        Ok(dcm_obj) => {
                            let meta_dcm = work_dcm::MetaDcm::from(
                                &dcm_obj,
                                &path.as_path().to_str().unwrap_or_default()
                            );
                            add_dcm_in_contents(&contents, &meta_dcm);

                            let new_save_in = create_new_path(&meta_dcm, save_in);
                            let mut dcm_obj = dcm_obj;
                            work_dcm::depersonalize_obj(&mut dcm_obj);
                            work_dcm::save_dcm(&dcm_obj, &new_save_in).unwrap_or_else(|e| {
                                eprintln!("Error saving depersonalized dicom [path: {}]: \n {:?} ",
                                          &path.as_path().to_str().unwrap_or_default(), e );
                            });
                        }
                        Err(_) => {}
                    };
                });
            export_res_as_json(contents);
        }
        Err(error) => {
            eprintln!("Error create data base in memory: {:?}", error)
        }
    }
}

fn create_new_path(meta_dcm: &work_dcm::MetaDcm, save_in: &path::PathBuf) -> String {
    let mut patient_id: &String = &"NoPatientID".to_string();
    let mut study_uid: &String = &"NoStudyDateTime".to_string();
    let mut series_uid: &String = &"NoSeriesUid".to_string();
    if meta_dcm.get_patient_ref().patient_id.len() != 0 {
        patient_id = &meta_dcm.get_patient_ref().patient_id
    }
    if meta_dcm.get_study_ref().study_uid.len() != 0 {
        study_uid = &meta_dcm.get_study_ref().study_uid;

    }
    if meta_dcm.get_series_ref().series_uid.len() != 0 {
        series_uid = &meta_dcm.get_series_ref().series_uid
    }
    let folder_for_mane_is_err = save_in
        .join(path::Path::new("out_data_dcm_finder"));

    let new_path = save_in
        .join(path::Path::new(patient_id.trim()))
        .join(path::Path::new(study_uid.trim()))
        .join(path::Path::new(series_uid.trim()));
    let new_path = match std::fs::create_dir_all(&new_path) {
        Ok(_) => { new_path }
        Err(_) => {
            std::fs::create_dir_all(&folder_for_mane_is_err).unwrap_or_else(|_|{});
            folder_for_mane_is_err
        }
    };
    let count_files = count_files_in_dir(&new_path);
    let file_name = path::Path::new(count_files.as_str());

    new_path.join(path::Path::new(file_name)).to_str().unwrap().trim().to_string()
}

/// Возвращает количество файлов в директории (без захода в подкаталоги)
fn count_files_in_dir(path: &path::PathBuf) -> String {
    match  fs::read_dir(path) {
        Ok(dir_entry) => {dir_entry.into_iter().count().to_string()}
        Err(_) => {0.to_string()}
    }
}

fn export_res_as_json(con: Arc<Mutex<work_db::Connection>>) {
    match con.lock() {
        Ok(c) => {
            c.export_result().unwrap_or_else(|e| {
                eprintln!("Error exporting result as json: {:?}", e);
            });
        }
        Err(e) => {
            eprintln!("Error exporting result as json: {:?}", e);
        }
    }
}

fn add_dcm_in_contents(con: &Arc<Mutex<work_db::Connection>>, meta_dcm: &work_dcm::MetaDcm) {
    match con.lock() {
        Ok(c) => {
            c.insert_dcm(&meta_dcm);
        }
        Err(e) => {
            eprintln!("Error insert meta dcm in db: {:?}", e);
        }
    }
}