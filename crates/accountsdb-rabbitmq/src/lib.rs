//! Solana `accountsdb` plugin adapter for the `metaplex-indexer` RabbitMQ
//! transport

#![deny(
    clippy::suspicious,
    clippy::style,
    missing_debug_implementations,
    missing_copy_implementations
)]
#![warn(clippy::pedantic, clippy::cargo, missing_docs)]

pub(crate) use solana_accountsdb_plugin_interface::accountsdb_plugin_interface as interface;

mod plugin;

pub use plugin::AccountsDbPluginRabbitMq;

#[no_mangle]
#[allow(improper_ctypes_definitions)]
/// Construct a new instance of the plugin.
///
/// # Safety
/// This function is only safe if called by a Solana `accountsdb` plugin manager
/// conformant to the plugin interface.
pub unsafe extern "C" fn _create_plugin() -> *mut dyn interface::AccountsDbPlugin {
    Box::into_raw(Box::new(AccountsDbPluginRabbitMq::default()))
}
