use std::borrow::Cow;
use std::ops;

use crate::data::Data;
use crate::entities::Empty;
use crate::entities::account::Account;
use crate::entities::attachment::Attachment;
use crate::entities::filter::Filter;
use crate::entities::instance::*;
use crate::entities::notification::Notification;
use crate::entities::push::Subscription;
use crate::entities::relationship::Relationship;
use crate::entities::report::Report;
use crate::entities::search_result::SearchResult;
use crate::entities::search_result::SearchResultV2;
use crate::entities::status::Emoji;
use crate::entities::status::Status;
use crate::errors::Error;
use crate::errors::Result;
use crate::event_stream::EventReader;
use crate::event_stream::WebSocket;
use crate::media_builder::MediaBuilder;
use crate::page::Page;
use crate::requests::AddFilterRequest;
use crate::requests::AddPushRequest;
use crate::requests::StatusesRequest;
use crate::requests::UpdateCredsRequest;
use crate::requests::UpdatePushRequest;
use crate::status_builder::NewStatus;
use crate::util::deserialise_blocking;

use futures::future::TryFutureExt;
use reqwest::Response;
use reqwest::RequestBuilder;
use reqwest::Client;

/// Your mastodon application client, handles all requests to and from Mastodon.
#[derive(Clone, Debug)]
pub struct Mastodon {
    pub(crate) client: Client,
    /// Raw data about your mastodon instance.
    pub data: Data,
}

macro_rules! gen_id_route {
    ($method:ident, $name:ident, $routetype:ty) => {
        /// Access Route `$routetype::ROUTE`
        ///
        /// Equivalent to `get(format!("/api/v1/{}/{}", $routetype::ROUTE, id))`
        ///
        /// # Errors
        ///
        /// If `access_token` is not set.
        ///
        /// ```no_run
        /// # extern crate elefren;
        /// # use elefren::prelude::*;
        /// # fn main() -> Result<(), Box<::std::error::Error>> {
        /// # let data = Data {
        /// #     base: \"https://example.com\".into(),
        /// #     client_id: \"taosuah\".into(),
        /// #     client_secret: \"htnjdiuae\".into(),
        /// #     redirect: \"https://example.com\".into(),
        /// #     token: \"tsaohueaheis\".into(),
        /// # };
        /// let client = Mastodon::from(data);
        /// let account = client.$name("42")?;
        /// #   Ok(())
        /// # }
        /// ```"
        pub async fn $name(&self, id: &str) -> Result<<$routetype as crate::routes::IdRoute>::Output> {
            self.$method::<$routetype>(id).await
        }
    }
}

impl Mastodon {
    async fn get<T: for<'de> serde::Deserialize<'de>>(&self, url: String) -> Result<T> {
        self.send(self.client.get(&url)).and_then(deserialise_blocking).await
    }

    async fn post<T: for<'de> serde::Deserialize<'de>>(&self, url: String) -> Result<T> {
        self.send(self.client.post(&url)).and_then(deserialise_blocking).await
    }

    async fn delete<T: for<'de> serde::Deserialize<'de>>(&self, url: String) -> Result<T> {
        self.send(self.client.delete(&url)).and_then(deserialise_blocking).await
    }

    fn route(&self, url: &str) -> String {
        format!("{}{}", self.base, url)
    }

    pub(crate) async fn send(&self, req: RequestBuilder) -> Result<Response> {
        let request = req.bearer_auth(&self.token).build()?;
        self.client.execute(request)
            .await
            .map_err(Error::from)
    }

    paged_routes! {
        (get) favourites: "favourites" => Status,
        (get) blocks: "blocks" => Account,
        (get) domain_blocks: "domain_blocks" => String,
        (get) follow_requests: "follow_requests" => Account,
        (get) get_home_timeline: "timelines/home" => Status,
        (get) get_local_timeline: "timelines/public?local=true" => Status,
        (get) get_federated_timeline: "timelines/public?local=false" => Status,
        (get) get_emojis: "custom_emojis" => Emoji,
        (get) mutes: "mutes" => Account,
        (get) notifications: "notifications" => Notification,
        (get) reports: "reports" => Report,
        (get (q: &'a str, #[serde(skip_serializing_if = "Option::is_none")] limit: Option<u64>, following: bool,)) search_accounts: "accounts/search" => Account,
        (get) get_endorsements: "endorsements" => Account,
    }

    paged_routes_with_id! {
        (get) followers: "accounts/{}/followers" => Account,
        (get) following: "accounts/{}/following" => Account,
        (get) reblogged_by: "statuses/{}/reblogged_by" => Account,
        (get) favourited_by: "statuses/{}/favourited_by" => Account,
    }

    route! {
        (delete (domain: String,)) unblock_domain: "domain_blocks" => Empty,
        (get) instance: "instance" => Instance,
        (get) verify_credentials: "accounts/verify_credentials" => Account,
        (post (account_id: &str, status_ids: Vec<&str>, comment: String,)) report: "reports" => Report,
        (post (domain: String,)) block_domain: "domain_blocks" => Empty,
        (post (id: &str,)) authorize_follow_request: "accounts/follow_requests/authorize" => Empty,
        (post (id: &str,)) reject_follow_request: "accounts/follow_requests/reject" => Empty,
        (get  (q: &'a str, resolve: bool,)) search: "search" => SearchResult,
        (post (uri: Cow<'static, str>,)) follows: "follows" => Account,
        (post) clear_notifications: "notifications/clear" => Empty,
        (post (id: &str,)) dismiss_notification: "notifications/dismiss" => Empty,
        (get) get_push_subscription: "push/subscription" => Subscription,
        (delete) delete_push_subscription: "push/subscription" => Empty,
        (get) get_filters: "filters" => Vec<Filter>,
        (get) get_follow_suggestions: "suggestions" => Vec<Account>,
    }

    route_v2! {
        (get (q: &'a str, resolve: bool,)) search_v2: "search" => SearchResultV2,
    }

    /// Generic function for making a GET request to "{self.base}/api/v1/{Route::ROUTE}/{id}"
    ///
    /// # Returns
    ///
    /// Result of Route::OUTPUT
    ///
    #[inline]
    async fn route_get_id<Route: crate::routes::IdGetRoute>(&self, id: &str) -> Result<Route::Output> {
        let route = format!("{}/api/v1/{}/{}", self.base, Route::ROUTE, id);
        self.get(route).await
    }

    /// Generic function for making a POST request to "{self.base}/api/v1/{Route::ROUTE}/{id}"
    ///
    /// # Returns
    ///
    /// Result of Route::OUTPUT
    ///
    #[inline]
    async fn route_post_id<Route: crate::routes::IdPostRoute>(&self, id: &str) -> Result<Route::Output> {
        let route = format!("{}/api/v1/{}/{}", self.base, Route::ROUTE, id);
        self.post(route).await
    }

    /// Generic function for making a DELETE request to "{self.base}/api/v1/{Route::ROUTE}/{id}"
    ///
    /// # Returns
    ///
    /// Result of Route::OUTPUT
    ///
    #[inline]
    async fn route_delete_id<Route: crate::routes::IdDeleteRoute>(&self, id: &str) -> Result<Route::Output> {
        let route = format!("{}/api/v1/{}/{}", self.base, Route::ROUTE, id);
        self.delete(route).await
    }

    gen_id_route!(route_delete_id , delete_filter           , crate::routes::DeleteFilter);
    gen_id_route!(route_delete_id , delete_from_suggestions , crate::routes::DeleteFromSuggestions);
    gen_id_route!(route_delete_id , delete_status           , crate::routes::DeleteStatus);
    gen_id_route!(route_get_id    , get_account             , crate::routes::GetAccount);
    gen_id_route!(route_get_id    , get_card                , crate::routes::GetCard);
    gen_id_route!(route_get_id    , get_context             , crate::routes::GetContext);
    gen_id_route!(route_get_id    , get_filter              , crate::routes::GetFilter);
    gen_id_route!(route_get_id    , get_notification        , crate::routes::GetNotification);
    gen_id_route!(route_get_id    , get_status              , crate::routes::GetStatus);
    gen_id_route!(route_get_id    , mute                    , crate::routes::Mute);
    gen_id_route!(route_get_id    , unmute                  , crate::routes::Unmute);
    gen_id_route!(route_post_id   , block                   , crate::routes::Block);
    gen_id_route!(route_post_id   , endorse_user            , crate::routes::EndorseUser);
    gen_id_route!(route_post_id   , favourite               , crate::routes::Favourite);
    gen_id_route!(route_post_id   , follow                  , crate::routes::Follow);
    gen_id_route!(route_post_id   , reblog                  , crate::routes::Reblog);
    gen_id_route!(route_post_id   , unblock                 , crate::routes::Unblock);
    gen_id_route!(route_post_id   , unendorse_user          , crate::routes::UnendorseUser);
    gen_id_route!(route_post_id   , unfavourite             , crate::routes::Unfavourite);
    gen_id_route!(route_post_id   , unfollow                , crate::routes::Unfollow);
    gen_id_route!(route_post_id   , unreblog                , crate::routes::Unreblog);

    /// POST /api/v1/filters
    pub async fn add_filter(&self, request: &mut AddFilterRequest) -> Result<Filter> {
        let url = self.route("/api/v1/filters");
        let response = self.send(self.client.post(&url).json(&request)).await?;

        let status = response.status();

        if status.is_client_error() {
            return Err(Error::Client(status));
        } else if status.is_server_error() {
            return Err(Error::Server(status));
        }

        deserialise_blocking(response).await
    }

    /// PUT /api/v1/filters/:id
    pub async fn update_filter(&self, id: &str, request: &mut AddFilterRequest) -> Result<Filter> {
        let url = self.route(&format!("/api/v1/filters/{}", id));
        let response = self.send(self.client.put(&url).json(&request)).await?;

        let status = response.status();

        if status.is_client_error() {
            return Err(Error::Client(status));
        } else if status.is_server_error() {
            return Err(Error::Server(status));
        }

        deserialise_blocking(response).await
    }

    /// Update credentials
    pub async fn update_credentials(&self, builder: UpdateCredsRequest) -> Result<Account> {
        let changes = builder.build()?;
        let url = self.route("/api/v1/accounts/update_credentials");
        let response = self.send(self.client.patch(&url).json(&changes)).await?;

        let status = response.status();

        if status.is_client_error() {
            return Err(Error::Client(status));
        } else if status.is_server_error() {
            return Err(Error::Server(status));
        }

        deserialise_blocking(response).await
    }

    /// Post a new status to the account.
    pub async fn new_status(&self, status: NewStatus) -> Result<Status> {
        let response = self.send(
            self.client
                .post(&self.route("/api/v1/statuses"))
                .json(&status),
        ).await?;

        deserialise_blocking(response).await
    }

    /// Get timeline filtered by a hashtag(eg. `#coffee`) either locally or
    /// federated.
    pub async fn get_hashtag_timeline<'a>(&'a self, hashtag: &str, local: bool) -> Result<Page<'a, Status>> {
        let base = "/api/v1/timelines/tag/";
        let url = if local {
            self.route(&format!("{}{}?local=1", base, hashtag))
        } else {
            self.route(&format!("{}{}", base, hashtag))
        };

        let response = self.send(self.client.get(&url)).await?;
        Page::new(self, response).await
    }

    /// Get statuses of a single account by id. Optionally only with pictures
    /// and or excluding replies.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # extern crate elefren;
    /// # use elefren::prelude::*;
    /// # use std::error::Error;
    /// # fn main() -> Result<(), Box<dyn Error>> {
    /// # let data = Data {
    /// #   base: "".into(),
    /// #   client_id: "".into(),
    /// #   client_secret: "".into(),
    /// #   redirect: "".into(),
    /// #   token: "".into(),
    /// # };
    /// let client = Mastodon::from(data);
    /// let statuses = client.statuses("user-id", None)?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// ```no_run
    /// # extern crate elefren;
    /// # use elefren::prelude::*;
    /// # use std::error::Error;
    /// # fn main() -> Result<(), Box<dyn Error>> {
    /// # let data = Data {
    /// #   base: "".into(),
    /// #   client_id: "".into(),
    /// #   client_secret: "".into(),
    /// #   redirect: "".into(),
    /// #   token: "".into(),
    /// # };
    /// let client = Mastodon::from(data);
    /// let request = StatusesRequest::new()
    ///     .only_media();
    /// let statuses = client.statuses("user-id", request)?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn statuses<'a, 'b: 'a, S>(&'b self, id: &'b str, request: S) -> Result<Page<'b, Status>>
    where
        S: Into<Option<StatusesRequest<'a>>>,
    {
        let mut url = format!("{}/api/v1/accounts/{}/statuses", self.base, id);

        if let Some(request) = request.into() {
            url = format!("{}{}", url, request.to_querystring()?);
        }

        let response = self.send(self.client.get(&url)).await?;

        Page::new(self, response).await
    }

    /// Returns the client account's relationship to a list of other accounts.
    /// Such as whether they follow them or vice versa.
    pub async fn relationships<'a>(&'a self, ids: &[&str]) -> Result<Page<'a, Relationship>> {
        let mut url = self.route("/api/v1/accounts/relationships?");

        if ids.len() == 1 {
            url += "id=";
            url += ids[0];
        } else {
            for id in ids {
                url += "id[]=";
                url += id;
                url += "&";
            }
            url.pop();
        }

        let response = self.send(self.client.get(&url)).await?;

        Page::new(self, response).await
    }

    /// Add a push notifications subscription
    pub async fn add_push_subscription(&self, request: &AddPushRequest) -> Result<Subscription> {
        let request = request.build()?;
        let response = self.send(
            self.client
                .post(&self.route("/api/v1/push/subscription"))
                .json(&request),
        ).await?;

        deserialise_blocking(response).await
    }

    /// Update the `data` portion of the push subscription associated with this
    /// access token
    pub async fn update_push_data(&self, request: &UpdatePushRequest) -> Result<Subscription> {
        let request = request.build();
        let response = self.send(
            self.client
                .put(&self.route("/api/v1/push/subscription"))
                .json(&request),
        ).await?;

        deserialise_blocking(response).await
    }

    /// Get all accounts that follow the authenticated user
    pub async fn follows_me<'a>(&'a self) -> Result<Page<'a, Account>> {
        let me = self.verify_credentials().await?;
        self.followers(&me.id).await
    }

    /// Get all accounts that the authenticated user follows
    pub async fn followed_by_me<'a>(&'a self) -> Result<Page<'a, Account>> {
        let me = self.verify_credentials().await?;
        self.following(&me.id).await
    }

    /// returns events that are relevant to the authorized user, i.e. home
    /// timeline & notifications
    ///
    /// # Example
    ///
    /// ```no_run
    /// # extern crate elefren;
    /// # use elefren::prelude::*;
    /// # use std::error::Error;
    /// use elefren::entities::event::Event;
    /// # fn main() -> Result<(), Box<dyn Error>> {
    /// # let data = Data {
    /// #   base: "".into(),
    /// #   client_id: "".into(),
    /// #   client_secret: "".into(),
    /// #   redirect: "".into(),
    /// #   token: "".into(),
    /// # };
    /// let client = Mastodon::from(data);
    /// for event in client.streaming_user()? {
    ///     match event {
    ///         Event::Update(ref status) => { /* .. */ },
    ///         Event::Notification(ref notification) => { /* .. */ },
    ///         Event::Delete(ref id) => { /* .. */ },
    ///         Event::FiltersChanged => { /* .. */ },
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn streaming_user(&self) -> Result<EventReader<WebSocket>> {
        let mut url: url::Url = self.route("/api/v1/streaming").parse()?;
        url.query_pairs_mut()
            .append_pair("access_token", &self.token)
            .append_pair("stream", "user");
        let mut url: url::Url = reqwest::blocking::get(url.as_str())?
            .url()
            .as_str()
            .parse()?;
        let new_scheme = match url.scheme() {
            "http" => "ws",
            "https" => "wss",
            x => return Err(Error::Other(format!("Bad URL scheme: {}", x))),
        };
        url.set_scheme(new_scheme)
            .map_err(|_| Error::Other("Bad URL scheme!".to_string()))?;

        let client = tungstenite::connect(url.as_str())?.0;

        Ok(EventReader(WebSocket(client)))
    }

    /// returns all public statuses
    pub fn streaming_public(&self) -> Result<EventReader<WebSocket>> {
        let mut url: url::Url = self.route("/api/v1/streaming").parse()?;
        url.query_pairs_mut()
            .append_pair("access_token", &self.token)
            .append_pair("stream", "public");
        let mut url: url::Url = reqwest::blocking::get(url.as_str())?
            .url()
            .as_str()
            .parse()?;
        let new_scheme = match url.scheme() {
            "http" => "ws",
            "https" => "wss",
            x => return Err(Error::Other(format!("Bad URL scheme: {}", x))),
        };
        url.set_scheme(new_scheme)
            .map_err(|_| Error::Other("Bad URL scheme!".to_string()))?;

        let client = tungstenite::connect(url.as_str())?.0;

        Ok(EventReader(WebSocket(client)))
    }

    /// Returns all local statuses
    pub fn streaming_local(&self) -> Result<EventReader<WebSocket>> {
        let mut url: url::Url = self.route("/api/v1/streaming").parse()?;
        url.query_pairs_mut()
            .append_pair("access_token", &self.token)
            .append_pair("stream", "public:local");
        let mut url: url::Url = reqwest::blocking::get(url.as_str())?
            .url()
            .as_str()
            .parse()?;
        let new_scheme = match url.scheme() {
            "http" => "ws",
            "https" => "wss",
            x => return Err(Error::Other(format!("Bad URL scheme: {}", x))),
        };
        url.set_scheme(new_scheme)
            .map_err(|_| Error::Other("Bad URL scheme!".to_string()))?;

        let client = tungstenite::connect(url.as_str())?.0;

        Ok(EventReader(WebSocket(client)))
    }

    /// Returns all public statuses for a particular hashtag
    pub fn streaming_public_hashtag(&self, hashtag: &str) -> Result<EventReader<WebSocket>> {
        let mut url: url::Url = self.route("/api/v1/streaming").parse()?;
        url.query_pairs_mut()
            .append_pair("access_token", &self.token)
            .append_pair("stream", "hashtag")
            .append_pair("tag", hashtag);
        let mut url: url::Url = reqwest::blocking::get(url.as_str())?
            .url()
            .as_str()
            .parse()?;
        let new_scheme = match url.scheme() {
            "http" => "ws",
            "https" => "wss",
            x => return Err(Error::Other(format!("Bad URL scheme: {}", x))),
        };
        url.set_scheme(new_scheme)
            .map_err(|_| Error::Other("Bad URL scheme!".to_string()))?;

        let client = tungstenite::connect(url.as_str())?.0;

        Ok(EventReader(WebSocket(client)))
    }

    /// Returns all local statuses for a particular hashtag
    pub fn streaming_local_hashtag(&self, hashtag: &str) -> Result<EventReader<WebSocket>> {
        let mut url: url::Url = self.route("/api/v1/streaming").parse()?;
        url.query_pairs_mut()
            .append_pair("access_token", &self.token)
            .append_pair("stream", "hashtag:local")
            .append_pair("tag", hashtag);
        let mut url: url::Url = reqwest::blocking::get(url.as_str())?
            .url()
            .as_str()
            .parse()?;
        let new_scheme = match url.scheme() {
            "http" => "ws",
            "https" => "wss",
            x => return Err(Error::Other(format!("Bad URL scheme: {}", x))),
        };
        url.set_scheme(new_scheme)
            .map_err(|_| Error::Other("Bad URL scheme!".to_string()))?;

        let client = tungstenite::connect(url.as_str())?.0;

        Ok(EventReader(WebSocket(client)))
    }

    /// Returns statuses for a list
    pub fn streaming_list(&self, list_id: &str) -> Result<EventReader<WebSocket>> {
        let mut url: url::Url = self.route("/api/v1/streaming").parse()?;
        url.query_pairs_mut()
            .append_pair("access_token", &self.token)
            .append_pair("stream", "list")
            .append_pair("list", list_id);
        let mut url: url::Url = reqwest::blocking::get(url.as_str())?
            .url()
            .as_str()
            .parse()?;
        let new_scheme = match url.scheme() {
            "http" => "ws",
            "https" => "wss",
            x => return Err(Error::Other(format!("Bad URL scheme: {}", x))),
        };
        url.set_scheme(new_scheme)
            .map_err(|_| Error::Other("Bad URL scheme!".to_string()))?;

        let client = tungstenite::connect(url.as_str())?.0;

        Ok(EventReader(WebSocket(client)))
    }

    /// Returns all direct messages
    pub fn streaming_direct(&self) -> Result<EventReader<WebSocket>> {
        let mut url: url::Url = self.route("/api/v1/streaming").parse()?;
        url.query_pairs_mut()
            .append_pair("access_token", &self.token)
            .append_pair("stream", "direct");
        let mut url: url::Url = reqwest::blocking::get(url.as_str())?
            .url()
            .as_str()
            .parse()?;
        let new_scheme = match url.scheme() {
            "http" => "ws",
            "https" => "wss",
            x => return Err(Error::Other(format!("Bad URL scheme: {}", x))),
        };
        url.set_scheme(new_scheme)
            .map_err(|_| Error::Other("Bad URL scheme!".to_string()))?;

        let client = tungstenite::connect(url.as_str())?.0;

        Ok(EventReader(WebSocket(client)))
    }

    /// Equivalent to /api/v1/media
    pub async fn media(&self, media_builder: MediaBuilder) -> Result<Attachment> {
        use reqwest::multipart::{Form, Part};
        use std::{fs::File, io::Read};

        let mut f = File::open(media_builder.file.as_ref())?;
        let mut bytes = Vec::new();
        f.read_to_end(&mut bytes)?;
        let part = Part::stream(bytes);
        let mut form_data = Form::new().part("file", part);

        if let Some(description) = media_builder.description {
            form_data = form_data.text("description", description);
        }

        if let Some(focus) = media_builder.focus {
            let string = format!("{},{}", focus.0, focus.1);
            form_data = form_data.text("focus", string);
        }

        let response = self.send(
            self.client
                .post(&self.route("/api/v1/media"))
                .multipart(form_data),
        ).await?;

        let status = response.status();

        if status.is_client_error() {
            return Err(Error::Client(status));
        } else if status.is_server_error() {
            return Err(Error::Server(status));
        }

        deserialise_blocking(response).await
    }
}

impl From<Data> for Mastodon {
    /// Creates a mastodon instance from the data struct.
    fn from(data: Data) -> Mastodon {
        let mut builder = MastodonBuilder::default();
        builder.data(data);
        builder
            .build()
            .expect("We know `data` is present, so this should be fine")
    }
}

impl ops::Deref for Mastodon {
    type Target = Data;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

/// Builder to build a `Mastodon` object
#[derive(Debug)]
pub struct MastodonBuilder {
    client: Option<Client>,
    data: Option<Data>,
}

impl Default for MastodonBuilder {
    fn default() -> Self {
        MastodonBuilder {
            client: None,
            data: None,
        }
    }
}

impl MastodonBuilder {

    /// Set the client for the mastodon object to be built
    pub fn client(&mut self, client: Client) -> &mut Self {
        self.client = Some(client);
        self
    }

    /// Set the data for the mastodon object to be built
    pub fn data(&mut self, data: Data) -> &mut Self {
        self.data = Some(data);
        self
    }

    /// Build the `Mastodon` object
    pub fn build(self) -> Result<Mastodon> {
        Ok(if let Some(data) = self.data {
            Mastodon {
                client: self.client.unwrap_or_else(Client::new),
                data,
            }
        } else {
            return Err(Error::MissingField("missing field 'data'"));
        })
    }
}

