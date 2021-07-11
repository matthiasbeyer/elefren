use crate::entities::account::Account;

/// Equivalent to `get("/api/v1/{IdRoute::ROUTE}")
///
/// # Errors
///
/// If `access_token` is not set.
pub trait IdRoute {
    /// Route fragment appended after, e.g., "/api/v1/"
    const ROUTE: &'static str;

    /// Output of the route
    type Output: for<'de> serde::Deserialize<'de>;
}

/// Marker trait for GET request routes where an ID is passed
pub trait IdGetRoute: IdRoute { /* empty */ }

/// Access the "accounts/{}" endpoint
#[derive(Debug, Copy, Clone)]
pub struct GetAccount;
impl IdRoute for GetAccount {
    const ROUTE: &'static str = "accounts/{}";
    type Output = Account;
}
impl IdGetRoute for GetAccount {}

