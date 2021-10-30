use ring::digest::{Context, SHA256};
use data_encoding::HEXLOWER;
use chrono::{Utc, prelude::*};

use std::io::{self, Read, Write, BufReader, prelude::*};
use std::fs::{self, File};
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub enum Error{
    ArgErr,
    MissingTime,
    IoErr(std::io::Error),
}

impl From<std::io::Error> for Error{
    fn from(err: std::io::Error) -> Self {
        Error::IoErr(err)
    }
}

#[derive(Clone)]
pub struct FileData {
    pub path: String,
    pub hash: Option<String>,
}

impl FileData {
    pub fn from_path(path: &Path) -> FileData {
        FileData {
            path:  String::from(fs::canonicalize(path).unwrap().to_str().unwrap()),
            hash: FileData::hash(&path).ok(),
        }
    }

    pub fn to_string(&self) -> String {
        format!(
            "{},{}",
            self.path,
            self.print_hash()
        )
    }

    pub fn to_file(file_data: &Vec<FileData>, file: &mut File) -> io::Result<()> {
        
        write!(file, "{}\n", Utc::now())?;
        write!(file, "path,hash\n")?;
    
        for data in file_data {
            write!(file, "{}\n", data.to_string())?;
        }

        Ok(())
    }

    pub fn hash<P: AsRef<Path>>(path: &P) -> io::Result<String> {
        let mut context = Context::new(&SHA256);
        let mut buffer = [0; 1024];
        let mut file = File::open(path)?;
    
        loop {
            let count = file.read(&mut buffer)?;
            if count == 0 {
                break;
            }
            context.update(&buffer[..count]);
        }
    
        Ok(HEXLOWER.encode(context.finish().as_ref()))
    }

    pub fn print_hash(&self) -> &str {
        if let Some(hash) = &self.hash {
            &hash
        }
        else {
            "None"
        }
    }
}

fn help() {
    println!(
        "An operation argument must be passed.\n\
        \n\
        Operations:\n\
        \t-s [Directory]\n\
        \tScans the given [Directory] for executable files. If [Directory] is omitted \"/\" will be used. If -o is not used default name is \"scan.csv\".\n\
        \n\
        \t-c [Old File] [New File]\n\
        \tCompares the two files for changes. If -o is not used default name is \"compare.csv\".\n\
        \n\
        \t-h\n\
        \tDisplays this help menu.\n\
        \n\
        Other arguments:\n\
        \t-o [Name]\tSets the used output file. Can be used with -s and -c."
    );
}

enum ArgType {
    Scan(Option<String>),
    Compare((Option<String>, Option<String>)),
    Output(Option<String>),
    Help,
    Error(String),
}
use std::env::{Args, args};
use std::iter::Peekable;

impl ArgType {
    pub fn read_args() -> Result<Vec<ArgType>, Vec<String>> {
        let mut arg_types = Vec::new();
        let mut args = args().peekable();
        args.next();

        while let Some(arg) = args.next(){
            arg_types.push(ArgType::arg_type(arg, &mut args))
        }

        let mut errors = ArgType::collect_errors(&mut arg_types);

        if !errors.is_empty() {
            Err(errors)
        }
        else {
            if arg_types.is_empty() {
                errors.push(String::from("No args given."));
                Err(errors)
            }
            else {
                Ok(arg_types)
            }
        }
    }

    fn next_is_arg(arg: &mut Peekable<Args>) -> bool {
        if let Some(value) = arg.peek(){
            value.starts_with('-')
        }
        else {
            false
        }
    }

    fn arg_type(main: String, mut others: &mut Peekable<Args>) -> ArgType {
        use ArgType::*;

        let mut chars = main.chars();
        if chars.next() != Some('-') {return Error(main)}

        if let Some(arg) = chars.next() {
            match arg {
                's' => {
                    return if ArgType::next_is_arg(&mut others) {
                        Scan(None)
                    }
                    else {
                        Scan(others.next())
                    }
                },
                'c' => {
                    return Compare((others.next(), others.next()))
                },
                'o' => {
                    return Output(others.next())
                }
                'h' => {
                    return Help
                },
                _ => {
                    return Error(main)
                }
            }
        }
        
        Error(main)
    }

    pub fn is_err(&self) -> bool {
        use ArgType::*;

        if let Error(_) = self {
            true
        }
        else {
            false
        }
    }

    fn collect_errors(all: &mut Vec<ArgType>) -> Vec<String> {
        use ArgType::*;

        let mut errors = Vec::new();

        let mut i = 0;
        while i < all.len() {
            if all[i].is_err() {
                if let Error(err) = all.remove(i) {
                    errors.push(format!("Unknown argument: \"{}\".", err));
                }
            }
            else {
                i += 1;
            }
        }

        errors
    }
}

pub enum RunType {
    Scan(ScanArgs),
    Compare(CompArgs),
    Help,
}

impl RunType {
    pub fn new() -> Result<RunType, Vec<String>> {
        match ArgType::read_args() {
            Err(errs) => {Err(errs)},
            Ok(args) => {
                match RunType::find_values(&args) {
                    Err(_) => {Err(vec![String::from("No operation given.")])}
                    Ok((run_type, out)) => {
                        RunType::craft_type(args, run_type, out)
                    }
                }
            }
        }
    }

    fn craft_type(args: Vec<ArgType>, run_type: usize, out: Option<String>) -> Result<RunType, Vec<String>> {
        use RunType::*;

        let run_type = &args[run_type];

        match run_type {
            ArgType::Scan(path) => {
                Ok(Scan(ScanArgs {
                    dir: if let Some(path) = path {
                        PathBuf::from(path)
                    }
                    else {
                        PathBuf::from("/")
                    },
                    out: if let Some(out) = out {
                        PathBuf::from(out)
                    }
                    else {
                        PathBuf::from("scan.csv")
                    },
                }))
            },
            ArgType::Compare((old, new)) => {
                Ok(Compare(CompArgs {
                    old: if let Some(old) = old {
                        PathBuf::from(old)
                    }
                    else {
                        return Err(vec![String::from("Missing file location for compare.")])
                    },
                    new: if let Some(new) = new {
                        PathBuf::from(new)
                    }
                    else {
                        return Err(vec![String::from("Missing file location for compare.")])
                    },
                    out: if let Some(out) = out {
                        PathBuf::from(out)
                    }
                    else {
                        PathBuf::from("comp.csv")
                    },
                }))
            },
            ArgType::Help => Ok(Help),
            _ => Err(vec![String::from("Error Crafting run type.")])
        }
    }

    fn find_values(args: &Vec<ArgType>) -> Result<(usize, Option<String>), ()>{
        use ArgType::*;

        let mut out = None;
        let mut run_type = None;

        for (i, arg) in args.iter().enumerate() {
            match arg {
                Error(_) => (),
                Scan(_) => {
                    if run_type.is_none() {
                        run_type = Some(i);
                    }
                },
                Compare(_) => {
                    if run_type.is_none() {
                        run_type = Some(i);
                    }
                },
                Help => {
                    if run_type.is_none() {
                        run_type = Some(i);
                    }
                },
                Output(name) => {
                    if out.is_none() {
                        out = name.clone();
                    }
                },
            }

            if run_type.is_some() && out.is_some() {break}
        }

        if let Some(index) = run_type {
            Ok((index, out))
        }
        else {
            Err(())
        }
    }

    pub fn run(&self) -> Result<(), Error> {
        use RunType::*;

        match self {
            Help => {
                help();
                Ok(())
            },
            Scan(scan) => scan.run(),
            Compare(comp) => comp.run(),
        }
    }
}

/// Holds all of the data for running the scan mode.
pub struct ScanArgs {
    dir: PathBuf,
    out: PathBuf,
}

impl ScanArgs {
    pub fn run(&self) -> Result<(), Error> {
        let mut files = Vec::new();
        let mut out = File::create(&self.out)?;

        ScanArgs::scan_dir(&self.dir, &mut files).unwrap();

        files.sort_by(|a, b| a.path.cmp(&b.path));

        FileData::to_file(&files, &mut out).unwrap();

        Ok(())
    }

    fn scan_dir(path: &Path, files: &mut Vec<FileData>) -> io::Result<()> {
        if let Ok(dir) = path.read_dir() {
            println!("Scanning {}.", path.display());
            for child in dir {
                if let Ok(child) = child {
                    if child.file_type()?.is_dir() {
                        ScanArgs::scan_dir(&child.path(), files)?;
                    }
                    else if child.file_type()?.is_file() {
                        ScanArgs::check_file(&child.path(), files)?;
                    }
                }
            }
        }
        else {
            eprintln!("Failed to scan: {}.", path.display());
        }
        
        Ok(())
    }
    
    fn check_file(path: &Path, files: &mut Vec<FileData>) -> io::Result<()> {
        if ScanArgs::is_exe(&path)? {
            files.push(
                FileData::from_path(path)
            );
        }
    
        Ok(())
    }
    
    fn check_bit(value: u32, bit: u8) -> bool {
        (value & (1 << bit)) > 0
    }
    
    fn is_exe<P: AsRef<Path>>(path: &P) -> io::Result<bool> {
        use std::os::unix::fs::MetadataExt;

        let mode = std::fs::metadata(path)?.mode();
    
        //user x
        if ScanArgs::check_bit(mode, 6) {return Ok(true)}
        //group x
        if ScanArgs::check_bit(mode, 3) {return Ok(true)}
        //other x
        if ScanArgs::check_bit(mode, 0) {return Ok(true)}
    
        Ok(false)
    }
}

///Convert read strings into data structures without having to reallocate a string every time.
struct FileReader {
    file: BufReader<File>,
    string: String,
}

impl FileReader {
    pub fn new(path: &Path) -> io::Result<FileReader> {
        Ok(FileReader {
            file: BufReader::new(File::open(path)?),
            string: String::new()
        })
    }

    fn read(&mut self) -> io::Result<()> {
        self.string.clear();
        self.file.read_line(&mut self.string)?;
        Ok(())
    }

    pub fn skip(&mut self) {
        let _ = self.read();
    }

    pub fn read_time(&mut self) -> Result<DateTime<Utc>, Error>{
        self.read()?;

        if let Ok(time) = Utc.datetime_from_str(&self.string, "%Y-%m-%d %H:%M:%S.%f %Z\n"){
            Ok(time)
        }
        else {
            Err(Error::MissingTime)
        }
    }

    pub fn read_filedata(&mut self) -> Result<Option<FileData>, Error>{
        self.read()?;

        if self.string.is_empty() {return Ok(None)}

        let parts: Vec<&str> = self.string.split(',').collect();

        Ok(Some(FileData{
            path: String::from(parts[0]),
            hash: if parts[1] == "None" {
                None
            }
            else {
                Some(String::from(parts[1].trim()))
            }
        }))
    }
}

/// Holds all of the data for running the comparison mode.
pub struct CompArgs {
    old: PathBuf,
    new: PathBuf,
    out: PathBuf,
}

impl CompArgs {
    pub fn run(&self) -> Result<(), Error> {
        let mut old = FileReader::new(&self.old)?;
        let mut new = FileReader::new(&self.new)?;
        let mut out = File::create(&self.out)?;

        write!(out, "{}", CompArgs::time_header(&mut old, &mut new)?)?;
        write!(out, "Path,Change,Old Hash,New Hash\n")?;

        //skip header
        old.skip();
        new.skip();

        CompArgs::compare(&mut old, &mut new, &mut out)?;

        Ok(())
    }

    ///generates the time header for out file and swaps old and new if there times don't match up.
    fn time_header(old: &mut FileReader, new: &mut FileReader) -> Result<String, Error> {
        let mut old_time = old.read_time()?;
        let mut new_time = new.read_time()?;

        if new_time < old_time {
            std::mem::swap(&mut new_time, &mut old_time);
            std::mem::swap(new, old);
        }

        Ok(format!("{} to {}\n", old_time, new_time))
    }

    fn compare(old_reader: &mut FileReader, new_reader: &mut FileReader, out: &mut File) -> Result<(), Error> {
        use std::cmp::Ordering::*;

        let mut old_o = old_reader.read_filedata()?;
        let mut new_o = new_reader.read_filedata()?;

        //runs when only when there is a valid pair to compare.
        while let (Some(old), Some(new)) = (&old_o, &new_o) {
            match old.path.cmp(&new.path){
                Less => {
                    write!(out, "{},deleted,{}\n", old.path, old.print_hash())?;
                    old_o = old_reader.read_filedata()?;
                }
                Equal => {
                    if old.hash != new.hash {
                        write!(out, "{},updated,{},{}\n", old.path, old.print_hash(), new.print_hash())?;
                    }
                    old_o = old_reader.read_filedata()?;
                    new_o = new_reader.read_filedata()?;
                }
                Greater => {
                    write!(out, "{},created,,{}\n", new.path, new.print_hash())?;
                    new_o = new_reader.read_filedata()?;
                }
            }
        }

        //dump the rest of the old file marked as deleted.
        while let Some(old) = old_o {
            write!(out, "{},deleted\n", old.path)?;
            old_o = old_reader.read_filedata()?;
        }

        //dump the rest of the new file marked as created.
        while let Some(new) = new_o {
            write!(out, "{},created\n", new.path)?;
            new_o = new_reader.read_filedata()?;
        }

        Ok(())
    }
}