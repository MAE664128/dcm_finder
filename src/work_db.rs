pub use rusqlite::{Connection, Result, Error};
use rusqlite::NO_PARAMS;
use crate::work_dcm;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Write;



#[derive(Serialize, Deserialize,Debug)]
pub struct Pa {
    pub patient_id: String,
    pub birth_date: String,
    pub sex: String,
    pub age: String,
    pub studies: Vec<St>,
}

impl Pa {
    pub fn count_studies(&self) -> usize {
        self.studies.len()
    }

    pub fn count(&self) -> (usize, usize, usize) {
        let len_studies = self.count_studies();
        let mut len_series = 0;
        let mut len_paths = 0;
        for study in &self.studies {
            let (tmp_len_series, tmp_len_paths) = study.count();
            len_series = len_series + tmp_len_series;
            len_paths = len_paths + tmp_len_paths;
        };
        (len_studies, len_series, len_paths)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct St {
    pub study_uid: String,
    pub study_date: String,
    pub study_time: String,
    pub description: String,
    pub series: Vec<Se>,
}

impl St {
    pub fn count_series(&self) -> usize {
        self.series.len()
    }

    pub fn count(&self) -> (usize, usize){
        (self.count_series(), self.series.iter().map(|series|{series.count_paths()}).sum())
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Se {
    pub series_uid: String,
    pub modality: String,
    pub instancenumber: String,
    pub imagepositionpatient: String,
    pub imageorientationpatient: String,
    pub pixelspacing: String,
    pub numberofframes: String,
    pub xraytubecurrent: String,
    pub kvp: String,
    pub filtertype: String,
    pub rows: String,
    pub columns: String,
    pub exposuretime: String,
    pub rescaleintercept: String,
    pub description: String,
    pub paths: Vec<String>,
}

impl Se {
    pub fn count_paths(&self) -> usize {
        self.paths.len()
    }
}




pub trait Dcm {
    fn create_dcm_tables(in_memory: bool) -> Result<Connection, Error>;
    fn insert_dcm(&self, meta_dcm: &work_dcm::MetaDcm);
    fn insert_path(&self, path: &str) -> Result<(), Error>;
    fn insert_path_with_uid(&self, path: &str, series_uid: &String) -> Result<(), Error>;

    fn get_or_add_patient(&self, p: &work_dcm::MetaPatient) -> Result<String, Error>;
    fn get_or_add_study(&self, p: &work_dcm::MetaStudy, patient_id: &String) -> Result<String, Error>;
    fn get_or_add_series(&self, p: &work_dcm::MetaSeries, study_uid: &String) -> Result<String, Error>;
    fn get_patients_as_struct(&self) -> Result<Vec<Pa>, Error>;
    fn get_studies_as_struct(&self, patient_id: &String) -> Result<Vec<St>, Error>;
    fn get_series_as_struct(&self, study_uid: &String) -> Result<Vec<Se>, Error>;
    fn get_paths_as_vec(&self, series_uid: &String) -> Result<Vec<String>, Error>;
    fn print_count(vec_patients: &Vec<Pa>) -> Result<()>;
    fn export_result(&self) -> Result<(), Error>;
}

impl Dcm for Connection {
    /// Создает таблицы в sqlite
    fn create_dcm_tables(in_memory: bool) -> Result<Connection, Error> {
        let conn = if in_memory {
            Connection::open_in_memory()?
        } else {
            Connection::open("study.db")?
        };
        // Для использования даты необходимо соблюдать формат YYYY-MM-DD HH:MM:SS.SSS
        conn.execute(
            "CREATE TABLE IF NOT EXISTS patients (
                patient_id TEXT NOT NULL CHECK (length(patient_id) <= 64) PRIMARY KEY,
                birth_date TEXT DEFAULT NULL,
                sex TEXT DEFAULT NULL,
                age TEXT DEFAULT NULL
            );
        ",
            NO_PARAMS,
        )?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS study (
                study_uid TEXT NOT NULL CHECK (length(study_uid) <= 64) PRIMARY KEY,
                study_date TEXT DEFAULT NULL,
                study_time TEXT DEFAULT NULL,
                description TEXT DEFAULT NULL,
                patient_id TEXT NOT NULL,
                FOREIGN KEY (patient_id)
                REFERENCES patients (patient_id)
                ON UPDATE CASCADE
            );
        ",
            NO_PARAMS,
        )?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS series (
                series_uid TEXT NOT NULL DEFAULT 'UIDNotSet' CHECK (length(series_uid) <= 64) PRIMARY KEY,
                modality TEXT DEFAULT NULL,
                instancenumber TEXT DEFAULT NULL,
                imagepositionpatient TEXT DEFAULT NULL,
                imageorientationpatient TEXT DEFAULT NULL,
                pixelspacing TEXT DEFAULT NULL,
                numberofframes TEXT DEFAULT NULL,
                xraytubecurrent TEXT DEFAULT NULL,
                kvp TEXT DEFAULT NULL,
                filtertype TEXT DEFAULT NULL,
                rows TEXT DEFAULT NULL,
                columns TEXT DEFAULT NULL,
                exposuretime TEXT DEFAULT NULL,
                rescaleintercept TEXT DEFAULT NULL,
                description TEXT DEFAULT NULL,

                study_uid TEXT NOT NULL,
                FOREIGN KEY (study_uid)
                REFERENCES study (study_uid)
                ON UPDATE CASCADE
            );
        ",
            NO_PARAMS,
        )?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS paths (
                path TEXT NOT NULL PRIMARY KEY,

                series_uid TEXT NOT NULL DEFAULT 'UIDNotSet',
                FOREIGN KEY (series_uid)
                REFERENCES series (series_uid)
                ON UPDATE CASCADE
            );
        ",
            NO_PARAMS,
        )?;

        Ok(conn)
    }

    fn insert_path(&self, path: &str) -> Result<(), Error> {
        self.execute(
            "INSERT OR IGNORE INTO `paths` (path) VALUES(?1);",
            &[path],
        )?;
        Ok(())
    }
    fn insert_path_with_uid(&self, path: &str, series_uid: &String) -> Result<(), Error> {
        self.execute(
            "INSERT OR IGNORE INTO `paths` (path, series_uid) VALUES(?1,?2);",
            &[path, series_uid],
        )?;
        Ok(())
    }

    fn insert_dcm(&self, meta_dcm: &work_dcm::MetaDcm) {
        if !match self.get_or_add_patient(&meta_dcm.get_patient_ref()) {
            Ok(patient_id) => {
                match self.get_or_add_study(&meta_dcm.get_study_ref(), &patient_id) {
                    Ok(study_uid) => {
                        match self.get_or_add_series(&meta_dcm.get_series_ref(), &study_uid) {
                            Ok(series_uid) => {
                                if self.insert_path_with_uid(meta_dcm.get_path_ref(), &series_uid)
                                    .is_ok() {
                                    true
                                } else {
                                    false
                                }
                            }
                            Err(_) => { false }
                        }
                    }
                    Err(_) => { false }
                }
            }
            Err(_) => { false }
        } {
            self.insert_path(meta_dcm.get_path_ref()).unwrap();
        }
    }

    fn get_or_add_series(&self, p: &work_dcm::MetaSeries, study_uid: &String) -> Result<String, Error> {
        self.execute(
            "INSERT OR IGNORE INTO `series` \
             VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16);",
            &[&p.series_uid, &p.modality, &p.instancenumber, &p.imagepositionpatient,
                &p.imageorientationpatient, &p.pixelspacing, &p.numberofframes,
                &p.xraytubecurrent, &p.kvp, &p.filtertype, &p.rows, &p.columns,
                &p.exposuretime, &p.rescaleintercept, &p.description, &study_uid],
        )?;
        let result = self.query_row(
            "SELECT series_uid FROM series WHERE series_uid = (?1);",
            &[&p.series_uid],
            |row| row.get(0),
        )?;
        Ok(result)
    }

    fn get_or_add_study(&self, p: &work_dcm::MetaStudy, patient_id: &String) -> Result<String, Error> {
        self.execute(
            "INSERT OR IGNORE INTO `study` \
             VALUES(?1,?2,?3,?4,?5);",
            &[&p.study_uid, &p.study_date, &p.study_time, &p.description, &patient_id],
        )?;
        let result = self.query_row(
            "SELECT study_uid FROM study WHERE study_uid = (?1);",
            &[&p.study_uid],
            |row| row.get(0),
        )?;
        Ok(result)
    }

    fn get_or_add_patient(&self, p: &work_dcm::MetaPatient) -> Result<String, Error> {
        self.execute(
            "INSERT OR IGNORE INTO `patients` \
             VALUES(?1,?2,?3,?4);",
            &[&p.patient_id, &p.birth_date, &p.sex, &p.age],
        )?;
        let result = self.query_row(
            "SELECT patient_id FROM patients WHERE patient_id = (?1);",
            &[&p.patient_id],
            |row| row.get(0),
        )?;
        Ok(result)
    }

    fn get_studies_as_struct(&self, patient_id: &String) -> Result<Vec<St>, Error> {
        let mut stmt = self.prepare("SELECT * FROM study WHERE patient_id = (?1);")?;
        let mut rows = stmt.query(&[&patient_id])?;
        let mut studies: Vec<St> = Vec::new();
        while let Some(row) = rows.next()? {
            let study_uid = row.get(0)?;
            studies.push(
                St {
                    study_uid: row.get(0)?,
                    study_date: row.get(1)?,
                    study_time: row.get(2)?,
                    description: row.get(3)?,
                    series: self.get_series_as_struct(&study_uid)?,
                });
        }
        Ok(studies)
    }

    fn get_patients_as_struct(&self) -> Result<Vec<Pa>, Error> {
        let mut stmt = self.prepare("SELECT * FROM patients")?;
        let mut rows = stmt.query(NO_PARAMS)?;
        let mut items: Vec<String> = Vec::new();
        let mut patients: Vec<Pa> = Vec::new();
        while let Some(row) = rows.next()? {
            items.push(row.get(0)?);
            let patient_id = row.get(0)?;
            patients.push(
                Pa {
                    patient_id: row.get(0)?,
                    birth_date: row.get(1)?,
                    sex: row.get(2)?,
                    age: row.get(3)?,
                    studies: self.get_studies_as_struct(&patient_id)?,
                }
            );
        }
        Ok(patients)
    }

    fn get_series_as_struct(&self, study_uid: &String) -> Result<Vec<Se>, Error> {
        let mut stmt = self.prepare("SELECT * FROM series WHERE study_uid = (?1);")?;
        let mut rows = stmt.query(&[&study_uid])?;
        let mut series: Vec<Se> = Vec::new();
        while let Some(row) = rows.next()? {
            let series_uid = row.get(0)?;
            series.push(
                Se {
                    series_uid: row.get(0)?,
                    modality: row.get(1)?,
                    instancenumber: row.get(2)?,
                    imagepositionpatient: row.get(3)?,
                    imageorientationpatient: row.get(4)?,
                    pixelspacing: row.get(5)?,
                    numberofframes: row.get(6)?,
                    xraytubecurrent: row.get(7)?,
                    kvp: row.get(8)?,
                    filtertype: row.get(9)?,
                    rows: row.get(10)?,
                    columns: row.get(11)?,
                    exposuretime: row.get(12)?,
                    rescaleintercept: row.get(13)?,
                    description: row.get(14)?,
                    paths: self.get_paths_as_vec(&series_uid)?,
                });
        }
        Ok(series)
    }

    fn get_paths_as_vec(&self, series_uid: &String) -> Result<Vec<String>, Error> {
        let mut stmt = self.prepare("SELECT * FROM paths WHERE series_uid = (?1);")?;
        let mut rows = stmt.query(&[&series_uid])?;
        let mut paths: Vec<String> = Vec::new();
        while let Some(row) = rows.next()? {
            paths.push(row.get(0)?);
        }
        Ok(paths)
    }

    fn print_count(vec_patients: &Vec<Pa>) -> Result<()> {
        println!("Among them, patients were found: {}", &vec_patients.len());
        for (i, patient) in vec_patients.iter().enumerate() {
            let tmp_tuple = patient.count();
            println!("\t{}. {:>15}--->\t\tStudies:\t{},\tSeries:\t{},\tFiles:\t{}",
                     i+1, patient.patient_id, tmp_tuple.0, tmp_tuple.1, tmp_tuple.2)
        }
        Ok(())
    }

    fn export_result(&self) -> Result<()> {
        let mut dict = HashMap::new();
        dict.insert("result", self.get_patients_as_struct()?);
        match dict.get(&"result") {
            Some(vec_patients) => {
                Connection::print_count(vec_patients);
            },
            _ => {println!("No patients found")}
        };
        let j = serde_json::to_string(&dict).unwrap();
        let mut file = File::create("result.json").unwrap();
        file.write_all(j.as_bytes()).unwrap();
        Ok(())
    }
}
