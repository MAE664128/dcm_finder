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
                let meta_dcm = work_dcm::MetaDcm::from(&dcm_obj,&path.as_path().to_str().unwrap());
                contents.lock().unwrap().insert_dcm(& meta_dcm);
            }
            Err(_) => {}
        };
    });
    contents.lock().unwrap().export_result();
}