extern crate crypto;
extern crate tempdir;

use std::fmt;
use std::env;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process;
use std::io;

use tempdir::TempDir;

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

    let cmd_file = dir.join(md5);

    if cmd_file.is_file() {
        let mut stdin = std::fs::File::open(cmd_file).unwrap();
        let mut stdout = io::stdout();
        io::copy(&mut stdin, &mut stdout);
    }
    else {
        let tmp_dir = TempDir::new_in(dir.as_path(), "workdir").unwrap();
        let tmp_path = tmp_dir.path().join("work");
        let file = std::fs::File::create(&tmp_path).unwrap();

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
    
        std::fs::rename(&tmp_path, &cmd_file).expect(format!("renamed failed {:?} -> {:?}", &tmp_path, &cmd_file).as_ref());
    }
}
  
