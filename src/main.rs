/* Copyright (C) 2017
 * Benoît Dejean <bdejean@gmail.com>
 * Cyprien Le Pannérer <cyplp@free.fr>
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful, but
 * WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
 * General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program; if not, see <http://www.gnu.org/licenses/>
 * or write to the Free Software Foundation, Inc., 51 Franklin St,
 * Fifth Floor, Boston, MA 02110-1301 USA
 */



extern crate crypto;
extern crate tempdir;

use std::env;
use std::fmt;
use std::fs;
use std::io;
use std::path::{Path,PathBuf};
use std::process;
use std::time::{Duration, SystemTime};

use tempdir::TempDir;

use crypto::md5::Md5;
use crypto::digest::Digest;


const MAX_DAYS_DEFAULT : f32 = 7.0;


fn concat_args(args : &[String]) -> String {
//    args.remove(0);
    let joined = args.join("@@join@@");
    return joined;
}

fn hash(s : &str) -> String {
    let mut h = Md5::new();
    h.input_str(s);
    return h.result_str();
}

fn get_max_days() -> f32 {
    match env::var("CMD_CACHE_MAX_DAYS") {
        Ok(val) => return check_max_days(&val),
        Err(_) => return MAX_DAYS_DEFAULT
    }
}

fn check_max_days(s: &str) -> f32  {
    match s.parse::<f32>() {
        Ok(val) if val >= 0.0 => {return val;},
        _ => {return MAX_DAYS_DEFAULT;},
    }
}


fn check_or_create_dir() -> PathBuf {
    let home = env::var("HOME").expect("HOME not set!");

    let dir = Path::new(&home).join(".cmd_cache");

    if !dir.is_dir() {
        std::fs::create_dir(&dir).expect(&format!("can't create {:?}", &dir));
    }

    return dir;
}

fn check_file(file: &PathBuf) -> bool{
    let ok = file.is_file();

    if ! ok { return false; }
    
    let metadata = fs::metadata(file).unwrap();
    let file_time = metadata.modified().unwrap();
    let duration = Duration::from_secs((get_max_days() * 24.0 * 3600.0) as u64);
    return file_time + duration > SystemTime::now();

}

fn cmd_cache(args : &[String], output : &mut std::io::Write) {
    let joined = concat_args(&args);

    let md5 = hash(&joined);

    let dir = check_or_create_dir();

    let cmd_file = dir.join(md5);

    if !check_file(&cmd_file) {
        eprint!("# Really running {:?}\n", args);
        let tmp_dir = TempDir::new_in(dir.as_path(), "workdir").unwrap();
        let tmp_path = tmp_dir.path().join("work");
        let file = std::fs::File::create(&tmp_path).unwrap();

        let stdout = std::process::Stdio::from(file);

        let cmd = &args[0];

        let mut child = std::process::Command::new(cmd)
            .args(&args[1..args.len()])
            .stdin(std::process::Stdio::null())
            .stdout(stdout)
            .spawn()
            .expect(format!("failed to execute {:?}", args).as_ref());

        child.wait().expect(format!("failed to wait {:?}", args).as_ref());
    
        std::fs::rename(&tmp_path, &cmd_file).expect(format!("renamed failed {:?} -> {:?}", &tmp_path, &cmd_file).as_ref());
    }

    let mut stdin = std::fs::File::open(cmd_file).unwrap();
    io::copy(&mut stdin, output);
}

fn main() {
    let args : Vec<String> = std::env::args().skip(1).collect();
    cmd_cache(&args, &mut io::stdout());
}


#[cfg(test)]
mod test {

    use ::*;

    fn clean_env(key : &str) -> Option <String>{
        match env::var(key) {
            Ok(value) => {env::remove_var(key);
                      return Some(value);},
            _ => None
        }
    }

    fn restore_env(key : &str, value :Option <String>) {
        match value {
            Some(val) => { env::set_var(key, val);},
            None => { }
        }
    }
                
    #[test]
    fn test_concat_args() {
        let args = [String::from("foo")];
        assert_eq!("foo", concat_args(&args));
        let args = [String::from("foo"), String::from("bar")];
        assert_eq!("foo@@join@@bar", concat_args(&args));
    }

    #[test]
    fn test_hash() {
        let s = "foo";
        assert_eq!(hash(&s), "acbd18db4cc2f85cedef654fccc4a4d8");
    }


    #[test]
    fn test_check_max_days() {
        assert_eq!(check_max_days("foo"), MAX_DAYS_DEFAULT);
        assert_eq!(check_max_days("-1"), MAX_DAYS_DEFAULT);
        assert_eq!(check_max_days("1a"), MAX_DAYS_DEFAULT);
        assert_eq!(check_max_days("1.03"), 1.03);
        assert_eq!(check_max_days("0"), 0 as f32);
        assert_eq!(check_max_days("6"), 6 as f32);
        assert_eq!(check_max_days("30"), 30 as f32);
    }

    #[test]
    fn test_cmd_cache() {
        let old_days = clean_env("CMD_CACHE_MAX_DAYS");
        env::set_var("CMD_CACHE_MAX_DAYS", "1");

        let old_home = clean_env("home");
        let tmp = TempDir::new("test_dir").unwrap();
        let home = tmp.path();
        env::set_var("HOME", home);
        
        let msg = "hello world";

        // never execute before
        let mut o : Vec<u8> = Vec::new();
        cmd_cache(&[String::from("echo"), String::from(msg)], &mut o);
        assert_eq!(msg.to_owned() + "\n", String::from_utf8(o).unwrap());

        // hits cache
        let mut o : Vec<u8> = Vec::new();
        cmd_cache(&[String::from("echo"), String::from(msg)], &mut o);
        assert_eq!(msg.to_owned() + "\n", String::from_utf8(o).unwrap());

        // cache too old
        env::set_var("CMD_CACHE_MAX_DAYS", "0");
        let mut o : Vec<u8> = Vec::new();
        cmd_cache(&[String::from("echo"), String::from(msg)], &mut o);
        assert_eq!(msg.to_owned() + "\n", String::from_utf8(o).unwrap());
        
        restore_env("CMD_CACHE_MAX_DAYS", old_days);
        restore_env("HOME", old_home);
    }
        
    #[test]
    fn test_get_max_days() {
        
        let old = clean_env("CMD_CACHE_MAX_DAYS");
        assert_eq!(get_max_days(), MAX_DAYS_DEFAULT);

        env::set_var("CMD_CACHE_MAX_DAYS", "foo");
        assert_eq!(get_max_days(), MAX_DAYS_DEFAULT);

        env::set_var("CMD_CACHE_MAX_DAYS", "-1");
        assert_eq!(get_max_days(), MAX_DAYS_DEFAULT);

        env::set_var("CMD_CACHE_MAX_DAYS", "0");
        assert_eq!(get_max_days(), 0.0);

        env::set_var("CMD_CACHE_MAX_DAYS", "1.0");
        assert_eq!(get_max_days(), 1.0);

        restore_env("CMD_CACHE_MAX_DAYS", old);
    }

    #[test]
    fn test_check_or_create_dir() {
        let old = clean_env("HOME");
        
        let tmp = TempDir::new("test_dir").unwrap();
        let home = tmp.path();

        env::set_var("HOME", home);
        
        let result = tmp.path().join(".cmd_cache"); 
        // .cmd_cache doesn't exists at this point.
        assert_eq!(check_or_create_dir(),
                   result);

        // .cmd_cache exists at this point.
        assert_eq!(check_or_create_dir(),
                   result);

        restore_env("HOME", old);
    }

    #[test]
    fn test_check_file() {
        let tmp = TempDir::new("test_dir").unwrap();
        let file = tmp.path().join("fake");

        assert_eq!(check_file(&file), false);

        let mut fileh = std::fs::File::create(&file).unwrap();
        
        let old = clean_env("CMD_CACHE_MAX_DAYS");

        env::set_var("CMD_CACHE_MAX_DAYS", "0");
        assert_eq!(check_file(&file), false);

        env::set_var("CMD_CACHE_MAX_DAYS", "1");
        assert_eq!(check_file(&file), true);
        
        restore_env("CMD_CACHE_MAX_DAYS", old);
    }
}
