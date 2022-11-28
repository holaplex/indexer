//! listing and offer upsert functions

/// functions to insert marketplace activity
pub mod activity;
/// Generic listing upsert function which returns listing uuid if upsert is successful
pub mod listing;
/// Generic offer upsert function which returns offer uuid if upsert is successful
pub mod offer;
/// Generic purchase upsert function which returns purchase uuid if upsert is successful
pub mod purchase;
