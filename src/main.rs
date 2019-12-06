use std::io::Write;

macro_rules! try_or_exit {
    ($expr: expr => $num:literal) => {
        match $expr {
            Ok(value) => value,
            Err(error) => {
                log::error!("{}", error);
                std::process::exit(-1)
            }
        }
    };
}

const LOG_LEVEL: &str = "info";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::from_env(env_logger::Env::default().default_filter_or(LOG_LEVEL))
        .format(|buf, record| writeln!(buf, "{}", record.args(),))
        .init();

    let builder = gh_auditor::AuditorBuilder::new("gh-audit-test");
    let mut auditor = try_or_exit!(builder.finish() => -1);

    if let Err(errors) = auditor.audit() {
        for err in errors {
            if err.is_audit() {
                log::error!("{}", err);
            } else {
                panic!("{}", err);
            }
        }

        std::process::exit(-2);
    }

    Ok(())
}
