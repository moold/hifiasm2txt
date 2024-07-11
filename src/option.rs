#![allow(dead_code)]
use clap::{value_parser, Arg, ArgAction, ArgMatches, Command};
use libc_stdhandle::stdout;
use path_absolutize::*;
use std::{
    ffi::CString,
    io::{Error, ErrorKind, Result},
    path::Path,
};

const VERSION: &str = include_str!(concat!(env!("OUT_DIR"), "/VERSION"));

#[derive(Clone, Debug)]
pub struct Option {
    pub thread: usize, //-t
    pub in_ec_bin: String,
    pub in_re_bin: String,
    pub in_so_bin: String,
    pub out_ec_bin: std::option::Option<String>, //-e
    pub out_re_bin: std::option::Option<String>, //-r
    pub out_so_bin: std::option::Option<String>, //-s
}

impl Option {
    pub fn new() -> Option {
        Option::default()
    }

    pub fn from_args() -> Option {
        let opt = Option::default();
        let args = Command::new("hifiasm2txt")
            .version(VERSION)
            .about(
                "A small tool to convert hifiasm bin files to text (gzip) format.",
            )
            .arg_required_else_help(true)
            .arg(
                Arg::new("hifiasm_prefix")
                    .value_name("PATH/PREFIX")
                    .value_parser(|x: &str| to_abspath_string(x, false))
                    .required(true)
                    .help("output file path and prefix of `hifiasm`."),
            )
            .arg(
                Arg::new("out_prefix")
                    .value_name("PREFIX")
                    .value_parser(|x: &str| to_abspath_string(x, false))
                    .default_value("hifiasm2txt")
                    .help("prefix of output files."),
            )
            .arg(
                Arg::new("thread")
                    .short('t')
                    .value_name("INT")
                    .default_value(opt.thread.to_string())
                    .value_parser(value_parser!(usize))
                    .hide(true)
                    .help("number of threads."),
            )
            .arg(
                Arg::new("out_ec_bin")
                    .short('e')
                    .long("out_ec_bin")
                    .help("convert PATH/PREFIX.ec.bin file.")
                    .action(ArgAction::SetTrue),
            )
            .arg(
                Arg::new("out_re_bin")
                    .short('r')
                    .long("out_re_bin")
                    .help("convert PATH/PREFIX.reverse.bin file.")
                    .action(ArgAction::SetTrue),
            )
            .arg(
                Arg::new("out_so_bin")
                    .short('s')
                    .long("out_so_bin")
                    .help("convert PATH/PREFIX.source.bin file.")
                    .action(ArgAction::SetTrue),
            )
            .get_matches();

        opt.update(args)
    }

    fn update(self, mut args: ArgMatches) -> Option {
        let in_prefix = args
            .remove_one::<String>("hifiasm_prefix")
            .expect("Missing --hifiasm_prefix!");
        let out_prefix = args
            .remove_one::<String>("out_prefix")
            .expect("Missing --out_prefix!");
        Option {
            in_ec_bin: to_abspath_string(format!("{}.ec.bin", in_prefix), true).unwrap(),
            in_re_bin: to_abspath_string(format!("{}.ovlp.reverse.bin", in_prefix), true).unwrap(),
            in_so_bin: to_abspath_string(format!("{}.ovlp.source.bin", in_prefix), true).unwrap(),
            out_ec_bin: if args.get_flag("out_ec_bin") {
                Some(to_abspath_string(format!("{}.ec.fasta.gz", out_prefix), false).unwrap())
            //safe
            } else {
                None
            },
            out_re_bin: if args.get_flag("out_re_bin") {
                Some(to_abspath_string(format!("{}.reverse.paf.gz", out_prefix), false).unwrap())
            //safe
            } else {
                None
            },
            out_so_bin: if args.get_flag("out_so_bin") {
                Some(to_abspath_string(format!("{}.source.paf.gz", out_prefix), false).unwrap())
            //safe
            } else {
                None
            },
            thread: args.remove_one::<usize>("thread").unwrap(),
            ..Default::default()
        }
    }
}

impl Default for Option {
    fn default() -> Self {
        Option {
            thread: 3,
            in_ec_bin: String::new(),
            in_re_bin: String::new(),
            in_so_bin: String::new(),
            out_ec_bin: None,
            out_re_bin: None,
            out_so_bin: None,
        }
    }
}

fn to_abspath_string<P: AsRef<Path>>(path: P, check_exist: bool) -> Result<String> {
    let path = path
        .as_ref()
        .absolutize()
        .expect("Failed convert input file to abspath!");
    if path.exists() || !check_exist {
        Ok(path.to_string_lossy().to_string())
    } else {
        Err(Error::new(
            ErrorKind::NotFound,
            format!("{:?} does not exist!", path),
        ))
    }
}

fn freopen_stdout(path: &str) -> Result<()> {
    let path = Path::new(path)
        .absolutize()
        .expect("Failed convert input file to abspath!");
    if path.exists() {
        return Err(Error::new(
            ErrorKind::AlreadyExists,
            format!("{:?} already exists!", path),
        ));
    } else {
        let w = CString::new("w").unwrap();
        let p = CString::new(path.to_string_lossy().as_bytes()).unwrap();
        if unsafe { libc::freopen(p.as_ptr(), w.as_ptr(), stdout()) }.is_null() {
            return Err(Error::new(
                ErrorKind::Other,
                format!("Failed to freopen: {:?}", path),
            ));
        }
    }
    Ok(())
}
