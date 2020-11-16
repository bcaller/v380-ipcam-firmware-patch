#[macro_use]
extern crate clap;

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use clap::AppSettings;
use md5::{Md5, Digest};
use std::fs;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::io::SeekFrom;
use std::path::Path;
use std::str;
use tempfile::tempfile;


fn main() -> io::Result<()> {
    let matches = clap_app!(patcher =>
        (version: "1.0")
        (author: "Ben Caller <https://github.com/bcaller>")
        (name: "v380 Patcher")
        (about: "Patch WiFi Smart Net Camera v380")
        (@subcommand read =>
            (about: "List files in patch, optionally extracting contents")
            (@arg PATCH_FILE: * +takes_value "Patch file to read")
            (@arg extract_dir: -e +takes_value "Directory to extract files to")
        )
        (@subcommand write =>
            (about: "Write a patch file")
            (@arg INPUT: ... * +takes_value "Files to add to patch")
            (@arg hwname: -h +takes_value "hwname of device")
        )
    )
    .setting(AppSettings::SubcommandRequired)
    .get_matches();
    if let Some(subm) = matches.subcommand_matches("read") {
        read(
            subm.value_of("PATCH_FILE").unwrap(),
            subm.value_of("extract_dir"),
        )?;
    } else if let Some(subm) = matches.subcommand_matches("write") {
        write(
            subm.values_of("INPUT").unwrap().collect(),
            subm.value_of("hwname").unwrap_or("V380E2_C"),
        )?;
    }
    Ok(())
}

fn filename_to_type(filename: &str) -> &str {
    match &filename[..3] {
        "IMG" => match &filename[4..7] {
            "KER" => "Kernel Image",
            "RFS" => "mtd1 Image",
            "USR" => "mtd2 Image",
            "MVS" => "mtd3 Image",
            "EXT" => "mtd4 Image",
            "JFS" => "mtd5 Image",
            _ => "Unknown Image",
        },
        "sf_" => "Sound",
        "exs" if &filename[3..4] == "h" => "Script",
        _ => "Other",
    }
}

pub fn read(patch_file: &str, extract_dir: Option<&str>) -> io::Result<()> {
    let mut f = File::open(&patch_file)?;
    f.seek(SeekFrom::Start(0x18))?;
    let n_sections = f.read_u32::<LittleEndian>()?;
    f.seek(SeekFrom::Start(0x80))?;
    println!("Number of files: {}", n_sections);
    for _i in 0..n_sections {
        let mut filename_buffer = vec![0; 0x38];
        f.read_exact(&mut filename_buffer)?;
        let null_terminated = filename_buffer.splitn(2, |c| *c == 0u8).next().unwrap();
        let filename = str::from_utf8(&null_terminated).unwrap();
        let length = f.read_u32::<LittleEndian>()?;
        f.seek(SeekFrom::Current(0x04))?;
        println!("{} ({} bytes, {})", filename, length, filename_to_type(filename));
        match extract_dir {
            Some(dir) => {
                let path = Path::new(dir).join(filename);
                let mut extract_f = File::create(path)?;
                let mut adapter = f.take(length as u64);
                io::copy(&mut adapter, &mut extract_f)?;
                f = adapter.into_inner();
            }
            _ => {
                f.seek(SeekFrom::Current(length as i64))?;
            }
        }
    }
    Ok(())
}

fn write_zeroes(f: &mut dyn Write, n: usize) -> io::Result<()> {
    let zero = vec![0; n];
    f.write_all(&zero)
}

fn write_string(f: &mut dyn Write, s: &str, n: usize) -> io::Result<()> {
    let s = s.as_bytes();
    f.write_all(s)?;
    write_zeroes(f, n - s.len())?;
    Ok(())
}

pub fn write(input: Vec<&str>, hwname: &str) -> io::Result<()> {
    let mut f = HashingWriter::new(tempfile()?);

    f.write_u32::<LittleEndian>(0x0a)?;
    write_string(&mut f, hwname, 0x10)?;
    f.write_u32::<LittleEndian>(0x1f4b59)?; // This might need to be configurable
    let n_files = input.len() as u32;
    f.write_u32::<LittleEndian>(n_files)?;
    write_zeroes(&mut f, 0x80 - 0x1c)?;

    for filename in input {
        let filesize = fs::metadata(filename)?.len() as u32;
        let basename = Path::new(filename).file_name().unwrap().to_str().unwrap();
        write_string(&mut f, basename, 0x38)?;
        f.write_u32::<LittleEndian>(filesize)?;
        write_zeroes(&mut f, 4)?;

        let mut input_f = File::open(filename)?;
        io::copy(&mut input_f, &mut f)?;
    }

    let digest = f.hash.result();
    let filename = format!("{:x}.patch", digest);
    // Copy temporary file as it may be on different disk
    let mut f = f.underlying_writer;
    f.seek(SeekFrom::Start(0))?;
    io::copy(&mut f, &mut File::create(filename)?)?;
    println!("{:x}", digest);
    Ok(())
}

struct HashingWriter<W: Write> {
    underlying_writer: W,
    hash: Md5,
}

impl<W: Write> HashingWriter<W> {
    pub fn new(wtr: W) -> HashingWriter<W> {
        HashingWriter {
            underlying_writer: wtr,
            hash: Md5::new(),
        }
    }
}

impl<W: Write> Write for HashingWriter<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.hash.input(buf);
        self.underlying_writer.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.underlying_writer.flush()
    }
}
