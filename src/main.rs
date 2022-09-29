use std::path::PathBuf;
use structopt::StructOpt;

//use arr::Array; //only nightly
//use copyless::BoxHelper;

use openssl::hash::{Hasher, MessageDigest};

use indicatif::{MultiProgress, ProgressBar, ProgressStyle};

//use bytes::{BufMut, BytesMut};
use std::fs::File;
use std::io::prelude::*;
use std::thread;
use std::io::BufReader;

#[derive(StructOpt)]
struct Opt {
    #[structopt(name("PATH"), parse(from_os_str), required(true))]
    paths: Vec<PathBuf>,
}

pub fn to_hex(blob: &[u8]) -> String {
    let mut buf = String::new();
    for ch in blob {
        fn hex_from_digit(num: u8) -> char {
            if num < 10 {
                (b'0' + num) as char
            } else {
                (b'a' + num - 10) as char
            }
        }
        buf.push(hex_from_digit(ch / 16));
        buf.push(hex_from_digit(ch % 16));
    }
    buf
}

const BUF_SIZE: u64 = 512000;

fn main() {
    //let mut buffer = BytesMut::with_capacity(102400);
    //let mut buffer: Box<[u8; BUF_SIZE]> = Box::new([0; BUF_SIZE]);
    //let mut buffer: Box<[u8; 1024000]> = Box::alloc().init([0; 1024000]);
    //let mut buffer: Array<u8> = Array::new(1024000);

    let args = Opt::from_args();
    //println!("Paths: {:?}", args.paths);

    let mut threads = vec![];
    let m = MultiProgress::new();

    for path in args.paths {
        if path.is_file() {
            //print!("Found file: {:?}", path.file_name().unwrap());
            let f = File::open(&path).unwrap();
            let pb = m.add(ProgressBar::new(f.metadata().unwrap().len() / BUF_SIZE));
            let sty = ProgressStyle::default_bar()
                .template("{prefix} {wide_msg:>} {bar:>50} {percent:>3}%")
                .progress_chars("■■-");
            threads.push(thread::spawn(move || {
                pb.set_draw_delta(f.metadata().unwrap().len() / BUF_SIZE / 200);
                let mut fr = BufReader::with_capacity(BUF_SIZE as usize, f);

                //print!("Found file: {:?}", path.file_name().unwrap());
                pb.set_style(sty);
                pb.set_prefix(path.file_name().unwrap().to_str().unwrap());
                pb.set_message("...");

                let mut hasher = Hasher::new(MessageDigest::md5()).unwrap();

                loop {
                    let buf_size = {
                        let buf = fr.fill_buf().expect("Unable to read data");
                        if buf.is_empty() {
                            break;
                        }
                        hasher.update(buf).unwrap();
                        pb.inc(1);
                        buf.len()
                    };
                    fr.consume(buf_size);
                }

                /*
                loop {
                    let n = f.read(&mut buffer[..]).unwrap();
                    if n == 0 {
                        break;
                    }
                    hasher.update(&buffer[..n]).unwrap();
                    pb.inc(1);
                }
                */

                let hex = hasher.finish().unwrap();
                pb.finish_with_message(&to_hex(hex.as_ref()));
                //println!(" {:?}", hex);
            }));
        } else {
            println!("Not a file: {:?}", path.file_name().unwrap());
        }
    }
    m.join().unwrap();
    //for th in threads {
    //    th.join().unwrap();
    //}
}
