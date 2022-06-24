use clap::{App,AppSettings,Arg,ArgMatches,SubCommand};

use article_server_rs::auth::{generate_random_string,hash_password};

pub fn setup_args() -> ArgMatches<'static> {
    App::new("webpage-saver-utils 1.0.0")
        .setting(AppSettings::ArgRequiredElseHelp)
        .subcommand(SubCommand::with_name("hash-password")
            .arg(Arg::with_name("password")
                .required(true)
                .help("Password to hash")))
        .subcommand(SubCommand::with_name("generate-random-string"))
    .get_matches()
}

fn main() {
    let args = setup_args();
    match args.subcommand() {
        ("hash-password", Some(args)) => {
            let password = args.value_of("password").expect("Unable to get password");
            let hash = hash_password(password).expect("Unable to hash supplied password");
            println!("{}", hash);
        },
        ("generate-random-string", Some(_args)) => {
            let r = generate_random_string();
            let s = std::str::from_utf8(&r).expect("Unable to generate random string");
            println!("{}", s);
        },
        _ => unreachable!()
    }
}
