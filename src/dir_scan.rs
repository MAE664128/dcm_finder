extern crate indicatif;

use std::path;
use std::convert::TryInto;
use std::sync::{Arc, Mutex};
use walkdir::{DirEntry, WalkDir};
use indicatif::ParallelProgressIterator;
use rayon::iter::{ParallelIterator, IntoParallelRefIterator};
use crate::work_db;
use crate::work_db::Dcm;

use crate::work_dcm;
use std::ffi::OsStr;
use rayon::join_context;

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
    let conn = work_db::Connection::create_dcm_tables(true).unwrap();
    let contents = Arc::new(Mutex::new(conn));
    paths.par_iter().progress_count(paths.len().try_into().unwrap()).for_each(|path| {
        match work_dcm::read_dcm(&path.as_path()) {
            Ok(dcm_obj) => {
                let meta_dcm = work_dcm::MetaDcm::from(&dcm_obj, &path.as_path().to_str().unwrap());
                contents.lock().unwrap().insert_dcm(&meta_dcm);
            }
            Err(_) => {}
        };
    });
    contents.lock().unwrap().export_result();
}

/// Выполняет рекурсивный поиск всех DICOM файлов в директории с
/// параллельной итерацией при выполнении операции чтения
/// Поиск не выполняется в скрытых директориях
pub fn de_identification_dcm_files(find_in: &path::PathBuf, save_in: &path::PathBuf) {
    let save_in = save_in.to_str().unwrap();
    let paths = find_all_files(find_in);
    println!("Total files found: {}", paths.len());
    let conn = work_db::Connection::create_dcm_tables(true).unwrap();
    let contents = Arc::new(Mutex::new(conn));
    paths.par_iter().progress_count(paths.len().try_into().unwrap()).for_each(|path| {
        match work_dcm::read_dcm(&path.as_path()) {
            Ok(dcm_obj) => {
                let meta_dcm = work_dcm::MetaDcm::from(&dcm_obj, &path.as_path().to_str().unwrap());
                contents.lock().unwrap().insert_dcm(&meta_dcm);
                let save_in= create_new_path(meta_dcm, save_in);
                let mut dcm_obj = dcm_obj;
                work_dcm::depersonalize_obj(&mut dcm_obj);


                work_dcm::save_dcm(&dcm_obj, save_in);
            }
            Err(_) => {}
        };
    });
    contents.lock().unwrap().export_result();
}

fn create_new_path(meta_dcm: work_dcm::MetaDcm, save_in: &str) -> String {
    let mut patient_id: &String = &"NoPatientID".to_string();
    let mut study_uid: &String = &"NoStudyUID".to_string();
    let mut series_uid: &String = &"NoSeriesUid".to_string();
    let mut file_name: &OsStr;
    if meta_dcm.get_patient_ref().patient_id.len() != 0 {
        patient_id = &meta_dcm.get_patient_ref().patient_id
    }
    if meta_dcm.get_study_ref().study_uid.len() == 0 {
        study_uid = &meta_dcm.get_study_ref().study_uid
    }
    if meta_dcm.get_series_ref().series_uid.len() == 0 {
        series_uid = &meta_dcm.get_series_ref().series_uid
    }
    file_name =  path::Path::new(meta_dcm.get_path_ref()).file_stem().unwrap();
    let save_in = path::Path::new(save_in).to_path_buf();
    let new_path = save_in
        .join(path::Path::new(patient_id))
        .join(path::Path::new(study_uid))
        .join(path::Path::new(series_uid));
    std::fs::create_dir_all(&new_path);
    new_path.join(path::Path::new(file_name)).to_str().unwrap().trim().to_string()
}

