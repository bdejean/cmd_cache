extern crate crypto;

use std::env;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

use crypto::md5::Md5;
use crypto::digest::Digest;

const MAX_DAYS_DEFAULT : f32 = 7.0;


fn concat_args(mut args : Vec<String>) -> String {
    args.remove(0);
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
    let args: Vec<String> = env::args().collect();

    let joined = concat_args(args);
    println!("{}", joined);

    let md5 = hash(joined);
    println!("{}", md5);

    let max_days = get_max_days();
    println!("{}", max_days);

    check_or_create_dir();
}
