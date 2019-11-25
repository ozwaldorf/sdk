use crate::config::{cache, dfx_version};
use crate::lib::env::VersionEnv;
use crate::lib::error::DfxResult;
use crate::lib::message::UserMessage;
use clap::{App, ArgMatches, SubCommand};
use std::io::Write;

pub fn construct() -> App<'static, 'static> {
    SubCommand::with_name("list").about(UserMessage::CacheList.to_str())
}

pub fn exec<T>(env: &T, _args: &ArgMatches<'_>) -> DfxResult
where
    T: VersionEnv,
{
    let mut current_printed = false;
    let current_version = env.get_version();
    let mut all_versions = cache::list_versions()?;
    all_versions.sort();
    for version in all_versions {
        if current_version.eq(&version) {
            current_printed = true;
            // Same version, prefix with `*`.
            std::io::stderr().flush()?;
            print!("{}", version);
            std::io::stdout().flush()?;
            eprintln!(" *");
        } else {
            eprintln!("{}", version);
        }
    }

    if !current_printed {
        // The current version wasn't printed, so it's not in the cache.
        std::io::stderr().flush()?;
        print!("{}", dfx_version());
        std::io::stdout().flush()?;
        eprintln!(" [missing]");
    }

    Ok(())
}
