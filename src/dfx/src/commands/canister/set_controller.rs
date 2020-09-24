use crate::lib::environment::Environment;
use crate::lib::error::{DfxError, DfxResult};
use crate::lib::identity::identity_manager::IdentityManager;
use crate::lib::message::UserMessage;
use crate::lib::models::canister_id_store::CanisterIdStore;
use crate::lib::waiter::waiter_with_timeout;
use crate::util::expiry_duration;
use clap::{App, Arg, ArgMatches, SubCommand};
use ic_agent::ManagementCanister;
use ic_types::principal::Principal as CanisterId;
use tokio::runtime::Runtime;

pub fn construct() -> App<'static, 'static> {
    SubCommand::with_name("set-controller")
        .about(UserMessage::CallCanister.to_str())
        .arg(
            Arg::with_name("canister")
                .takes_value(true)
                .help("A canister's name or its canister id.")
                .long("canister")
                .required(true),
        )
        .arg(
            Arg::with_name("new-controller")
                .takes_value(true)
                .help("An identity's name or its principal.")
                .long("new-controller")
                .required(true),
        )
}

pub fn exec(env: &dyn Environment, args: &ArgMatches<'_>) -> DfxResult {
    let canister = args.value_of("canister").unwrap();
    let canister_id = match CanisterId::from_text(canister) {
        Ok(id) => id,
        Err(_) => CanisterIdStore::for_env(env)?.get(canister)?,
    };

    let new_controller = args.value_of("new-controller").unwrap();
    let controller_principal = match CanisterId::from_text(new_controller) {
        Ok(principal) => principal,
        Err(_) => IdentityManager::new(env)?
            .instantiate_identity_from_name(new_controller)?
            .sender()?,
    };

    let timeout = expiry_duration();

    let mgr = ManagementCanister::new(
        env.get_agent()
            .ok_or(DfxError::CommandMustBeRunInAProject)?,
    );

    let mut runtime = Runtime::new().expect("Unable to create a runtime");
    runtime.block_on(mgr.set_controller(
        waiter_with_timeout(timeout),
        &canister_id,
        &controller_principal,
    ))?;

    Ok(())
}
