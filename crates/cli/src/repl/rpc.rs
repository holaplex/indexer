use docbot::prelude::*;

#[derive(Docbot)]
pub enum RpcCommand {
    /// `stores [search...]`
    /// Search the storefront list, or print all stores
    ///
    /// # Arguments
    /// search: An optional search string to filter the store list
    GetStorefronts(Vec<String>),
}

pub fn handle(cmd: RpcCommand) -> super::Result {
    todo!()
}
