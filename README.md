# DICOM finder &emsp; 

**Finder of study in dicom(CT) format, with the function of de-identification.**

---

```commandline
USAGE:
    dcm_finder <SUBCOMMAND>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

SUBCOMMANDS:
    depersonalize    Depersonalize all found DICOM files in the directory and save them in the specified directory
    find             Search for DICOM files in directory
    help             Prints this message or the help of the given subcommand(s)
```

**Function:**

- Search for DICOM files in the specified directory
- De-identification DICOM files in the specified directory (WIP)
- Export metadata about found DICOM files to JSON format

**Find**

```commandline
USAGE:
    dcm_finder find --path <find_in>

OPTIONS:
    -p, --path <find_in>    Input the path to the directory to search for DICOM files in it
```

**Depersonalize**

```commandline
USAGE:
    dcm_study_store depersonalize --path <find_in> --save <save_in>

OPTIONS:
    -p, --path <find_in>    Input the path to the directory to search for DICOM files in it
    -s, --save <save_in>    Input the path to the directory where the de-identified DICOM files will be saved
```

Example:

*(AMD Ryzen 7 3700X 8-Core Processor Samsung SSD 970 EVO Plus 1TB)*
```commandline
H:\> .\timecmd dcm_study_store find -p C:\...\MedImg
Total files found: 140781
Among them, patients were found: 5                                                                                     1
        1.    NoPatientID --->          Studies:        483,    Series: 952,    Files:  15338
        2.    SVR_1786577 --->          Studies:        1,      Series: 1,      Files:  451
        3.    SVR_1516417 --->          Studies:        1,      Series: 6,      Files:  772
        4.    SVR_2068053 --->          Studies:        1,      Series: 2,      Files:  548
        5.    SVR_1686009 --->          Studies:        1,      Series: 3,      Files:  661
command took 0:0:12.97 (12.97s total)
```
```json
{
   "result":[
      {
         "patient_id":"******",
         "birth_date":"******",
         "sex":"M ",
         "age":"037Y",
         "studies":[
            {
               "study_uid":"1.3.12.2.1107.5.2.40.50233.30000015102206510863000000019",
               "study_date":"",
               "study_time":"131308.588000 ",
               "description":"l-spine^lss ",
               "series":[
                  {
                     "series_uid":"1.3.12.2.1107.5.2.40.50233.2015102213164638517022660.0.0.0",
                     "modality":"MR",
                     "imagepositionpatient":"-16.02235101685\\-131.56626889218\\182.7740699195 ",
                     "imageorientationpatient":"1.432E-12\\1\\-2.05098E-10\\0.0069813299977\\-2.05103E-10\\-0.9999756302188",
                     "pixelspacing":"0.72916668653488\\0.72916668653488 ",
                      ...
                     "rows":"384",
                     "columns":"384",
                     "paths":[
                        "C:\\...\\L_MRI_Data\\0127\\L-SPINE_LSS_20151022_131308_588000\\T2_TSE_SAG_384_0002\\T2_TSE_SAG__0127_001.ima",
                        ...
                        "C:\\...\\L_MRI_Data\\0127\\L-SPINE_LSS_20151022_131308_588000\\T2_TSE_SAG_384_0002\\T2_TSE_SAG__0127_015.ima"
                     ]
                  },...
```