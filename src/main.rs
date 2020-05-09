/* Copyright (C) 2017-2020
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



extern crate chrono;
extern crate crypto;
extern crate tempfile;

use std::env;
use std::fs;
use std::io;
use std::path::{Path,PathBuf};
use std::time::{Duration, SystemTime};

use tempfile::NamedTempFile;

use chrono::{Local, Utc, TimeZone};

use crypto::md5::Md5;
use crypto::digest::Digest;


const MAX_DAYS_DEFAULT : f32 = 7.0;


fn dirty_parse_system_time(t : &SystemTime) -> (u64, u64) {
    return dirty_parse_system_time_str(&format!("{:?}", t));
}

fn dirty_parse_system_time_str(t : &str) -> (u64, u64) {
    let tokens = t.split(|c| c == ' ' || c == '\t' || c == ',');
    let mut ts = tokens.filter_map(|x| x.parse::<u64>().ok());
    return (ts.next().unwrap(), ts.next().unwrap());
}

fn dirty_system_time_format(t : &SystemTime) -> String {
    let (sec, nsec) = dirty_parse_system_time(t);
    return Utc.timestamp(sec as i64, nsec as u32).with_timezone(&Local).to_string();
}


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


fn get_max_days(s: &str) -> f32  {
    match s.parse::<f32>() {
        Ok(val) if val >= 0.0 => {return val;},
        _ => {return MAX_DAYS_DEFAULT;},
    }
}


fn check_or_create_dir(home: &str) -> PathBuf {
    let dir = Path::new(&home).join(".cmd_cache");

    if !dir.is_dir() {
        std::fs::create_dir(&dir).expect(&format!("can't create {:?}", &dir));
    }

    return dir;
}

fn check_file(max_days: f32, file: &PathBuf) -> Option<SystemTime> {
    let ok = file.is_file();

    if ! ok { return None; }
    
    let metadata = fs::metadata(file).unwrap();
    let file_time = metadata.modified().unwrap();
    let duration = Duration::from_secs((max_days * 24.0 * 3600.0) as u64);
    if file_time + duration > SystemTime::now() {
        return Some(file_time);
    } else {
        return None;
    }
}

fn cmd_cache(args : &[String], home: &str, max_days: f32, output : &mut dyn std::io::Write) {
    let joined = concat_args(&args);

    let md5 = hash(&joined);

    let dir = check_or_create_dir(home);

    let cmd_file = dir.join(md5);

    match check_file(max_days, &cmd_file) {
        Some(ts) => {
                    eprint!("# using cached output from {}\n", dirty_system_time_format(&ts));
        }
        None => {
            eprint!("# Really running {:?}\n", args);
            let tmp = NamedTempFile::new_in(dir.as_path()).unwrap();

            // from() moves the File, and into_file() as well, so the trick is try_clone()
            let stdout = std::process::Stdio::from(tmp.as_file().try_clone().unwrap());

            let cmd = &args[0];

            let mut child = std::process::Command::new(cmd)
                .args(&args[1..args.len()])
                .stdin(std::process::Stdio::null())
                .stdout(stdout)
                .spawn()
                .expect(format!("failed to execute {:?}", args).as_ref());

            child.wait().expect(format!("failed to wait {:?}", args).as_ref());

            // need to prepare the error message before because tmp.persist moves tmp
            let error_message = format!("failed to rename {:?} -> {:?}", &tmp.path(), &cmd_file);
            tmp.persist(&cmd_file).expect(error_message.as_ref());
        }
    }

    let mut stdin = std::fs::File::open(cmd_file).unwrap();
    io::copy(&mut stdin, output).expect("failed to display command output");
}

fn main() {
    let env_max_days = env::var("CMD_CACHE_MAX_DAYS").unwrap_or_default();
    let max_days = get_max_days(&env_max_days);
    let home = env::var("HOME").expect("HOME not set!");
    let args : Vec<String> = std::env::args().skip(1).collect();
    cmd_cache(&args, &home, max_days, &mut io::stdout());
}


#[cfg(test)]
mod test {

    extern crate rand;

    use tempfile::tempdir;
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
    fn test_get_max_days() {
        assert_eq!(get_max_days(""), MAX_DAYS_DEFAULT);
        assert_eq!(get_max_days("-1"), MAX_DAYS_DEFAULT);
        assert_eq!(get_max_days("0"), 0.0);
        assert_eq!(get_max_days("1.0"), 1.0);
        assert_eq!(get_max_days("1.03"), 1.03);
        assert_eq!(get_max_days("1a"), MAX_DAYS_DEFAULT);
        assert_eq!(get_max_days("30"), 30.0);
        assert_eq!(get_max_days("6"), 6.0);
        assert_eq!(get_max_days("foo"), MAX_DAYS_DEFAULT);

        use test::rand::{thread_rng, Rng};
        let d = thread_rng().gen_range::<f32>(0.0, 42.0);
        assert_eq!(get_max_days(&format!("{}", d)), d);
    }

    #[test]
    fn test_cmd_cache() {

        let tmp = tempdir().unwrap();
        let home = tmp.path().to_str().unwrap();
        
        let msg = "hello world";

        // never execute before
        let mut o : Vec<u8> = Vec::new();
        cmd_cache(&[String::from("echo"), String::from(msg)], home, 1.0, &mut o);
        assert_eq!(msg.to_owned() + "\n", String::from_utf8(o).unwrap());

        // hits cache
        let mut o : Vec<u8> = Vec::new();
        cmd_cache(&[String::from("echo"), String::from(msg)], home, 1.0, &mut o);
        assert_eq!(msg.to_owned() + "\n", String::from_utf8(o).unwrap());

        // cache too old
        let mut o : Vec<u8> = Vec::new();
        cmd_cache(&[String::from("echo"), String::from(msg)], home, 0.0, &mut o);
        assert_eq!(msg.to_owned() + "\n", String::from_utf8(o).unwrap());
        
        use test::rand::{thread_rng, Rng, distributions::Alphanumeric};
        let mut o : Vec<u8> = Vec::new();
        let v : String = thread_rng().sample_iter(&Alphanumeric).take(42).collect();
        cmd_cache(&[String::from("echo"), v.to_owned()], home, 0.0, &mut o);
        assert_eq!(v + "\n", String::from_utf8(o).unwrap());
    }


    #[test]
    fn test_check_or_create_dir() {
        let tmp = tempdir().unwrap();
        let home = tmp.path().to_str().unwrap();

        let result = tmp.path().join(".cmd_cache"); 
        // .cmd_cache doesn't exists at this point.
        assert_eq!(check_or_create_dir(home), result);

        // .cmd_cache exists at this point.
        assert_eq!(check_or_create_dir(home), result);
    }

    #[test]
    fn test_check_file() {
        let tmp = tempdir().unwrap();
        let file = tmp.path().join("fake");
        let _fileh = std::fs::File::create(&file).unwrap();
        
        assert!(check_file(0.0, &file).is_none());

        assert!(check_file(1.0, &file).is_some());
    }

    #[test]
    fn test_dirty_parse_system_time() {
        let s_now = SystemTime::now();
        let now = dirty_parse_system_time(&s_now);
        assert!(now.0 > 1530818304);
        assert_eq!(format!("{:?}", s_now), format!("SystemTime {{ tv_sec: {}, tv_nsec: {} }}", now.0, now.1));


        let (x_0, x_1) = (1530549407, 795369636);
        let x = dirty_parse_system_time_str(&format!("{{ tv_sec: {}, tv_nsec: {} }}", x_0, x_1));
        assert_eq!(x.0, x_0);
        assert_eq!(x.1, x_1);

    }
}
