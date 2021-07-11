use crate::entities::Empty;
use crate::entities::account::Account;
use crate::entities::card::Card;
use crate::entities::context::Context;
use crate::entities::filter::Filter;
use crate::entities::notification::Notification;
use crate::entities::relationship::Relationship;
use crate::entities::status::Status;

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

/// Marker trait for POST request routes where an ID is passed
pub trait IdPostRoute: IdRoute { /* empty */ }

/// Marker trait for DELETE request routes where an ID is passed
pub trait IdDeleteRoute: IdRoute { /* empty */ }

macro_rules! gen_route_type {
    ($t:ident, $marker:ty, ROUTE = $route:literal, Output = $output:ty) => {
        /// Route type $t for $route route
        #[derive(Debug, Copy, Clone)]
        pub struct $t;
        impl IdRoute for $t {
            const ROUTE: &'static str = $route;
            type Output = $output;
        }
        impl $marker for $t {}
    }
}

gen_route_type!(Block                 , IdPostRoute   , ROUTE = "accounts/{}/block"       , Output = Relationship);
gen_route_type!(DeleteFilter          , IdDeleteRoute , ROUTE = "filters/{}"              , Output = Empty);
gen_route_type!(DeleteFromSuggestions , IdDeleteRoute , ROUTE = "suggestions/{}"          , Output = Empty);
gen_route_type!(DeleteStatus          , IdDeleteRoute , ROUTE = "statuses/{}"             , Output = Empty);
gen_route_type!(EndorseUser           , IdPostRoute   , ROUTE = "accounts/{}/pin"         , Output = Relationship);
gen_route_type!(Favourite             , IdPostRoute   , ROUTE = "statuses/{}/favourite"   , Output = Status);
gen_route_type!(Follow                , IdPostRoute   , ROUTE = "accounts/{}/follow"      , Output = Relationship);
gen_route_type!(GetAccount            , IdGetRoute    , ROUTE = "accounts/{}"             , Output = Account);
gen_route_type!(GetCard               , IdGetRoute    , ROUTE = "statuses/{}/card"        , Output = Card);
gen_route_type!(GetContext            , IdGetRoute    , ROUTE = "statuses/{}/context"     , Output = Context);
gen_route_type!(GetFilter             , IdGetRoute    , ROUTE = "filters/{}"              , Output = Filter);
gen_route_type!(GetNotification       , IdGetRoute    , ROUTE = "notifications/{}"        , Output = Notification);
gen_route_type!(GetStatus             , IdGetRoute    , ROUTE = "statuses/{}"             , Output = Status);
gen_route_type!(Mute                  , IdGetRoute    , ROUTE = "accounts/{}/mute"        , Output = Relationship);
gen_route_type!(Reblog                , IdPostRoute   , ROUTE = "statuses/{}/reblog"      , Output = Status);
gen_route_type!(Unblock               , IdPostRoute   , ROUTE = "accounts/{}/unblock"     , Output = Relationship);
gen_route_type!(UnendorseUser         , IdPostRoute   , ROUTE = "accounts/{}/unpin"       , Output = Relationship);
gen_route_type!(Unfavourite           , IdPostRoute   , ROUTE = "statuses/{}/unfavourite" , Output = Status);
gen_route_type!(Unfollow              , IdPostRoute   , ROUTE = "accounts/{}/unfollow"    , Output = Relationship);
gen_route_type!(Unmute                , IdGetRoute    , ROUTE = "accounts/{}/unmute"      , Output = Relationship);
gen_route_type!(Unreblog              , IdPostRoute   , ROUTE = "statuses/{}/unreblog"    , Output = Status);

