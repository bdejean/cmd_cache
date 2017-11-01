extern crate crypto;

use std::env;

use crypto::md5::Md5;
use crypto::digest::Digest;

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

fn main() {
    let args: Vec<String> = env::args().collect();

    let joined = concat_args(args);
    println!("{}", joined);

    let md5 = hash(joined);

    println!("{}", md5);
}
