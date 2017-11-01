extern crate crypto;
extern crate tempfile;

use std::fmt;
use std::env;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process;

use crypto::md5::Md5;
use crypto::digest::Digest;


const MAX_DAYS_DEFAULT : f32 = 7.0;


fn concat_args(args : &Vec<String>) -> String {
//    args.remove(0);
    let joined = args.join("@@join@@");
    return joined;
}

fn hash(s :String) -> String {
    let mut h = Md5::new();
    h.input_str(&s);
    return h.result_str();
}

fn get_max_days() -> f32 {
    match env::var("CMD_CACHE_MAX_DAYS") {
        Ok(val) => return check_max_days(val),
        Err(_) => return MAX_DAYS_DEFAULT
    }
}

fn check_max_days(s: String) -> f32  {
    match s.parse::<f32>() {
        Ok(val) => {
            if val >= 0.0 {return val}
            return MAX_DAYS_DEFAULT;
        },
        Err(_) => return MAX_DAYS_DEFAULT,
    }
}


fn check_or_create_dir() -> PathBuf {
    let home = env::var("HOME");
    if home.is_ok(){
        let dir = Path::new(&home.unwrap()).join(".cmd_cache");
        if !dir.is_dir() {
            match std::fs::create_dir(&dir) {
                Ok(_) => return dir,
                Err(e) => panic!("can't create {:?} : {}", dir, e),
            }
        }
        else {
            return dir;
        }
        
    }
    else {
        panic!("HOME not set !");
    }
}

fn main() {
    let mut args: Vec<String> = env::args().collect();
    args.remove(0);
    let joined = concat_args(&args);
    println!("{}", joined);

    let md5 = hash(joined);
    println!("{}", md5);

    let max_days = get_max_days();
    println!("{}", max_days);

    let dir = check_or_create_dir();

    let tmp = tempfile::NamedTempFile::new_in(dir.as_path()).unwrap();
    let tmp_path = tmp.path().to_owned();
    let file : std::fs::File = tmp.into();
    let stdout = std::process::Stdio::from(file);

    let cmd = &args[0];

    let mut child = std::process::Command::new(cmd)
        .args(&args[1..args.len()])
        .stdin(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .stdout(stdout)
        .spawn()
        .expect(format!("failed to execute {:?}", args).as_ref());

    child.wait().expect(format!("failed to wait {:?}", args).as_ref());

    let from = tmp_path;
    let to = dir.join(md5);
    std::fs::rename(&from, &to).expect(format!("renamed failed {:?} -> {:?}", &from, &to).as_ref());
}
  
