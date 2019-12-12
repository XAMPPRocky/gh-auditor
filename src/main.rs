use std::io::Write;

use clap::clap_app;

fn try_or_exit<T>(result: gh_auditor::Result<T>, num: i32) -> T {
    match result {
        Ok(value) => value,
        Err(error) => {
            log::error!("{}", error);
            std::process::exit(num)
        }
    }
}

const LOG_LEVEL: &str = "info";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = clap_app!(gh_auditor =>
        (author: "Erin P. <xampprocky@gmail.com>")
        (@arg organisation: +takes_value +required
        "GitHub Organisation to audit. Requires `admin:read` level permissions")
        (@arg token: -t --token +takes_value
        "GitHub authentication token.")
    )
    .get_matches();

    env_logger::from_env(env_logger::Env::default().default_filter_or(LOG_LEVEL))
        .format(|buf, record| writeln!(buf, "{}", record.args()))
        .init();

    let mut builder = gh_auditor::AuditorBuilder::new(matches.value_of("organisation").unwrap());

    if let Some(key) = matches.value_of("token") {
        builder = builder.auth_key(key);
    }

    let mut auditor = try_or_exit(builder.finish(), -1);

    if auditor.audit().is_err() {
        std::process::exit(-2);
    }

    Ok(())
}
