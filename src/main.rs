extern crate clap;
extern crate fit_reader;
extern crate chrono;

use std::time::{Instant};
use fit_reader::{
    Record, FitType, MesgNumField,
    read_fit, timestamp_as_datetime
};
use std::fs;
use std::io::{Write};
use std::path::{Path};
use clap::{Arg, App};
use chrono::TimeZone;
use std::collections::{BTreeMap};

fn process_file(filepath: &Path)
{
    println!("#### Open {}", filepath.to_str().unwrap());
    let timer = Instant::now();
    let file = &mut fs::File::open(filepath).unwrap();
    let output = &mut fs::File::create(filepath.with_extension("txt")).unwrap();
    let records = read_fit(file).unwrap();
    let elapsed = timer.elapsed();
    let mut num_records = 0;
    let mut num_laps = 0;
    let mut samples = BTreeMap::new();
    for record in records {
        // println!("{}", record);
        match record {
            Record::Normal(ref rec) => {
                let record_type = rec.message_type();
                if let Some(rt) = record_type {
                    match rt {
                        MesgNumField::FileId => {
                            for field in &rec.fields {
                                if field.0 == 4 {
                                    if let FitType::Uint32(v) = field.2 {
                                        println!("File Id: {}",
                                            timestamp_as_datetime(v));
                                    }
                                }
                            }
                        },
                        MesgNumField::Record => {
                            num_records += 1;
                            let mut dt = None;
                            let mut hr = None;
                            for field in &rec.fields {
                                if field.0 == 253 {
                                    if let FitType::Uint32(v) = field.2 {
                                        dt = Some(timestamp_as_datetime(v));
                                    }
                                }
                                if field.0 == 3 && !field.2.is_invalid() {
                                    if let FitType::Uint8(v) = field.2 {
                                        hr = Some(v)
                                    }
                                }
                            }
                            if let Some(dt) = dt {
                                if let Some(hr) = hr {
                                    samples.insert(dt, hr);
                                }
                            }
                        },
                        MesgNumField::Lap => {
                            num_laps += 1;
                        },
                        MesgNumField::Activity => {
                            let mut dt = None;
                            let mut lt = None;
                            for field in &rec.fields {
                                if field.0 == 253 {
                                    if let FitType::Uint32(v) = field.2 {
                                        dt = Some(timestamp_as_datetime(v));
                                    }
                                }
                                if field.0 == 5 {
                                    if let FitType::Uint32(v) = field.2 {
                                        lt = Some(timestamp_as_datetime(v));
                                    }
                                }
                            }
                            if let Some(dt) = dt {
                                if let Some(lt) = lt {
                                    let diff = (lt.timestamp() - dt.timestamp()) as i32;
                                    let local = chrono::FixedOffset::east(diff).timestamp(dt.timestamp(), 0);
                                    println!("Activity date time: {}", local.to_rfc3339());
                                }
                            }
                        },
                        MesgNumField::Event => (),
                        MesgNumField::Session => (),
                        MesgNumField::DeviceInfo => (),
                        MesgNumField::FileCreator => (),
                        MesgNumField::DeviceSettings => (), // field 1: UTC offset, 2: Time offset
                        MesgNumField::UserProfile => (),
                        MesgNumField::Sport => (),
                        MesgNumField::ZonesTarget => (),
                        _ => {
                            println!("{}", rec);
                        },
                    }
                }
            },
            Record::CompressedTime(ref rec) => {
                let record_type = rec.message_type();
                if let Some(rt) = record_type {
                    println!("Compressed Time: {:?}", rt);
                }
            }
            _ => {}
        }
    }
    let (start, _) = samples.iter().next().unwrap();
    for (dt, hr) in samples.iter() {
        let offset = dt.signed_duration_since(*start).num_seconds();
        writeln!(output, "{}, {}", offset, hr).unwrap();
    }
    println!("Number of records: {}", num_records);
    println!("Number of laps: {}", num_laps);
    println!("Read in {}.{:09} s", elapsed.as_secs(), elapsed.subsec_nanos());
}

fn main() {

    let matches = App::new("FIT parsing")
        .arg(Arg::with_name("file")
                .short("f")
                .long("file")
                .multiple(true)
                .takes_value(true))
        .get_matches();
    let mut files = vec![];
    if let Some(args) = matches.values_of_lossy("file") {
        files = args.into_iter().collect();
    }
    for file in files {
        println!("Process {}", file);
        let m = fs::metadata(&file).unwrap();
        if m.is_dir() {
            for entry in fs::read_dir(&file).unwrap() {
                let path = entry.unwrap().path();
                if path.is_file() {
                    process_file(&path);
                }
            }
        }
        else {
            process_file(Path::new(&file));
        }
    }
}
