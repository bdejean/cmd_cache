use std::env;

fn concat_args(mut args : Vec<String>) -> String {
    args.remove(0);
    let joined = args.join("@@join@@");
    return joined;
}

fn main() {
    let args: Vec<String> = env::args().collect();

    println!("{}", concat_args(args));
}
