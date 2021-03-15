use dicom::core::Tag;
use dicom::core::dicom_value;
use dicom::core::value as dcm_core_value;
use dicom::object::mem::{InMemElement};
use dicom::object::open_file as dcm_core_open_file;

use dicom::object::{DefaultDicomObject, Result};
use std::path;
use std::collections::HashMap;


pub struct MetaDcm<'a> {
    patient: MetaPatient,
    study: MetaStudy,
    series: MetaSeries,
    path: &'a str,
}

#[derive(Debug)]
pub struct MetaPatient {
    pub patient_id: String,
    pub birth_date: String,
    pub sex: String,
    pub age: String,
}

pub struct MetaStudy {
    pub study_uid: String,
    pub study_date: String,
    pub study_time: String,
    pub description: String,
}

pub struct MetaSeries {
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
}

impl MetaDcm<'_> {
    pub fn from<'a>(obj: &'a DefaultDicomObject, path: &'a str) -> MetaDcm<'a> {
        MetaDcm {
            patient: MetaPatient {
                patient_id: get_value_for_tag(obj, Tag(0x0010, 0x0020)),
                birth_date: get_value_for_tag(obj, Tag(0x0010, 0x0030)),
                sex: get_value_for_tag(obj, Tag(0x0010, 0x0040)),
                age: get_value_for_tag(obj, Tag(0x0010, 0x1010)),
            },
            study: MetaStudy {
                study_uid: get_value_for_tag(obj, Tag(0x0020, 0x000D)),
                study_date: get_value_for_tag(obj, Tag(0x0008, 0x0020)),
                study_time: get_value_for_tag(obj, Tag(0x0008, 0x0030)),
                description: get_value_for_tag(obj, Tag(0x0008, 0x1030)),
            },
            series: MetaSeries {
                series_uid: get_value_for_tag(obj, Tag(0x0020, 0x000E)),
                modality: get_value_for_tag(obj, Tag(0x0008, 0x0060)),
                instancenumber: get_value_for_tag(obj, Tag(0x0020, 0x0013)),
                imagepositionpatient: get_value_for_tag(obj, Tag(0x0020, 0x0032)),
                imageorientationpatient: get_value_for_tag(obj, Tag(0x0020, 0x0037)),
                pixelspacing: get_value_for_tag(obj, Tag(0x0028, 0x0030)),
                numberofframes: get_value_for_tag(obj, Tag(0x0028, 0x0008)),
                xraytubecurrent: get_value_for_tag(obj, Tag(0x0018, 0x1151)),
                kvp: get_value_for_tag(obj, Tag(0x0018, 0x0060)),
                filtertype: get_value_for_tag(obj, Tag(0x0018, 0x1160)),
                rows: get_value_for_tag(obj, Tag(0x0028, 0x0010)),
                columns: get_value_for_tag(obj, Tag(0x0028, 0x0011)),
                exposuretime: get_value_for_tag(obj, Tag(0x0018, 0x1150)),
                rescaleintercept: get_value_for_tag(obj, Tag(0x0028, 0x1052)),
                description: get_value_for_tag(obj, Tag(0x0080, 0x103E)),
            },
            path: path,
        }
    }
    pub fn get_patient_ref(&self) -> &MetaPatient { &self.patient }
    pub fn get_study_ref(&self) -> &MetaStudy { &self.study }
    pub fn get_series_ref(&self) -> &MetaSeries { &self.series }
    pub fn get_path_ref(&self) -> &str { &self.path }
}

fn get_value_for_tag(obj: &DefaultDicomObject, tag: Tag) -> String {
    match obj.element(tag) {
        Ok(el) => {
            String::from(el.value().to_str().unwrap())
        }
        Err(_) => { String::from("Unknown") }
    }
}

pub fn depersonalize_obj(obj: &mut DefaultDicomObject) {
    let mut tags_for_depersonalization: HashMap<Tag, &str> = HashMap::new();
    // Patient's Name Attribute
    tags_for_depersonalization.insert(Tag(0x0010, 0x0010), "Unknown Name");
    // Patient's Birth Date Attribute
    tags_for_depersonalization.insert(Tag(0x0010, 0x0030), "19000101");
    // Patient's Birth Time Attribute
    tags_for_depersonalization.insert(Tag(0x0010, 0x0032), "084545");
    // Person's Address Attribute
    tags_for_depersonalization.insert(Tag(0x0040, 0x1102), "Unknown Address");
    // Patient's Death Date in Alternative Calendar Attribute
    tags_for_depersonalization.insert(Tag(0x0010, 0x0034), "19000101");
    // Patient Comments Attribute
    tags_for_depersonalization.insert(Tag(0x0010, 0x4000), "Unknown Comments");
    // Person's Phone
    tags_for_depersonalization.insert(Tag(0x0040, 0x1102), "Unknown Phone");
    // Institution Address Attribute
    tags_for_depersonalization.insert(Tag(0x0008, 0x0081), "Unknown Address");
    // Institution Name Attribute
    tags_for_depersonalization.insert(Tag(0x0008, 0x0080), "Unknown Attribute");
    // Admission ID Attribute
    tags_for_depersonalization.insert(Tag(0x0038, 0x0010), "Unknown ID");
    // Device Serial Number Attribute
    tags_for_depersonalization.insert(Tag(0x0018, 0x1000), "Unknown Serial Number");
    // Device Description Attribute
    tags_for_depersonalization.insert(Tag(0x0050, 0x0020), "Unknown Description");

    for (&tag, &values) in tags_for_depersonalization.iter() {
        replace_element_in_dcm_obj(obj, tag, values);
    };
}

/// Изменяет значение тега  объекта на новое значение
/// В случаи отсутствия тега в объекте вставка не выполняется.
fn replace_element_in_dcm_obj(obj: &mut DefaultDicomObject, tag: Tag, values: &str) {
    match obj.element(tag) {
        Ok(element) => {
            let value_dcm = dicom_value!(Strs,  [values]);

            let value_dcm =  dcm_core_value::Value::from(value_dcm);
            // : DataElement<InMemDicomObject<StandardDataDictionary>, InMemFragment>
            let new_el = InMemElement::new(
                tag,
                element.vr(),
                value_dcm
            );
            obj.put(new_el).unwrap();
        },
        _ => { }
    };
}

pub fn save_dcm(obj: &DefaultDicomObject, save_in: &String) ->Result<()> {
    obj.write_to_file(path::Path::new(save_in.as_str()))?;
    Ok(())
}

pub fn read_dcm(path: &path::Path) -> Result<DefaultDicomObject> {
    dcm_core_open_file(path)
}



