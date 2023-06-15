use std::time::Duration;

use axum::{
    error_handling::HandleErrorLayer,
    response::{IntoResponse, Response},
    routing::{get, post},
    BoxError, Json, Router,
};
use http::{header, HeaderValue, StatusCode};
use tower::ServiceBuilder;
use tower_http::{limit::RequestBodyLimitLayer, trace::TraceLayer};

use crate::{
    commit, fetch, file,
    scripting::{self, ComputeResult, ScriptContent, ScriptingError, ScriptingParam},
    track, view, RepoConfig, SharedState,
};
impl From<&RepoConfig> for hyper_ast_cvs_git::multi_preprocessed::ProcessingConfig<&'static str> {
    fn from(value: &RepoConfig) -> Self {
        match value {
            RepoConfig::CppMake => Self::CppMake {
                limit: 3,
                dir_path: "src",
            },
            RepoConfig::JavaMaven => Self::JavaMaven {
                limit: 3,
                dir_path: "",
            },
        }
    }
}
impl From<RepoConfig> for hyper_ast_cvs_git::multi_preprocessed::ProcessingConfig<&'static str> {
    fn from(value: RepoConfig) -> Self {
        (&value).into()
    }
}

impl IntoResponse for ScriptingError {
    fn into_response(self) -> Response {
        let mut resp = Json(self).into_response();
        *resp.status_mut() = StatusCode::BAD_REQUEST;
        resp
    }
}

// #[axum_macros::debug_handler]
async fn scripting(
    axum::extract::Path(path): axum::extract::Path<ScriptingParam>,
    axum::extract::State(state): axum::extract::State<SharedState>,
    axum::extract::Json(script): axum::extract::Json<ScriptContent>,
) -> axum::response::Result<Json<ComputeResult>> {
    let r = scripting::simple(script, state, path)?;
    Ok(r)
}

pub fn scripting_app(_st: SharedState) -> Router<SharedState> {
    let scripting_service_config = ServiceBuilder::new()
        .layer(HandleErrorLayer::new(|e: BoxError| async move {
            dbg!(e);
        }))
        .load_shed()
        .concurrency_limit(16)
        .buffer(200)
        .rate_limit(10, Duration::from_secs(5))
        // .request_body_limit(1024 * 5_000 /* ~5mb */)
        .timeout(Duration::from_secs(10))
        .layer(TraceLayer::new_for_http());
    Router::new().route(
        "/script/github/:user/:name/:commit",
        post(scripting).layer(scripting_service_config.clone()), // .with_state(Arc::clone(&shared_state)),
    )
    // .route(
    //     "/script/gitlab/:user/:name/:commit",
    //     post(scripting).layer(scripting_service_config), // .with_state(Arc::clone(&shared_state)),
    // )
}

pub fn fetch_git_file(_st: SharedState) -> Router<SharedState> {
    let service_config = ServiceBuilder::new()
        .layer(HandleErrorLayer::new(|e: BoxError| async move {
            dbg!(e);
        }))
        .load_shed()
        .concurrency_limit(8)
        .buffer(20)
        .rate_limit(5, Duration::from_secs(1))
        // .request_body_limit(1024 * 5_000 /* ~5mb */)
        .timeout(Duration::from_secs(10))
        .layer(TraceLayer::new_for_http());
    Router::new().route(
        "/file/github/:user/:name/:commit/*file",
        get(file).layer(service_config.clone()), // .with_state(Arc::clone(&shared_state)),
    )
}

// #[axum_macros::debug_handler]
async fn file(
    axum::extract::Path(path): axum::extract::Path<file::FetchFileParam>,
    axum::extract::State(state): axum::extract::State<SharedState>,
) -> axum::response::Result<String> {
    dbg!(&path);
    file::from_hyper_ast(state, path).map_err(|err| err.into())
}

pub fn track_code_route(_st: SharedState) -> Router<SharedState> {
    let service_config = ServiceBuilder::new()
        .layer(HandleErrorLayer::new(|e: BoxError| async move {
            dbg!(e);
        }))
        .load_shed()
        .concurrency_limit(8)
        .buffer(20)
        .rate_limit(2, Duration::from_secs(2))
        // .request_body_limit(1024 * 5_000 /* ~5mb */)
        .timeout(Duration::from_secs(10))
        .layer(TraceLayer::new_for_http());
    Router::new()
        .route(
            "/track/github/:user/:name/:commit/*file",
            get(track_code).layer(service_config.clone()), // .with_state(Arc::clone(&shared_state)),
        )
        .route(
            "/track_at_path/github/:user/:name/:commit/*path",
            get(track_code_at_path).layer(service_config.clone()),
        )
        .route(
            "/track_at_path_with_changes/github/:user/:name/:commit/*path",
            get(track_code_at_path_with_changes).layer(service_config.clone()),
        )
}

// #[axum_macros::debug_handler]
async fn track_code(
    axum::extract::Path(path): axum::extract::Path<track::TrackingParam>,
    axum::extract::Query(query): axum::extract::Query<track::TrackingQuery>,
    axum::extract::State(state): axum::extract::State<SharedState>,
) -> impl IntoResponse {
    dbg!(&path);
    dbg!(&query);
    track::track_code(state, path, query)
}
async fn track_code_at_path(
    axum::extract::Path(path): axum::extract::Path<track::TrackingAtPathParam>,
    axum::extract::State(state): axum::extract::State<SharedState>,
    axum::extract::Query(query): axum::extract::Query<track::TrackingQuery>,
) -> impl IntoResponse {
    dbg!(&path);
    dbg!(&query);
    track::track_code_at_path(state, path, query)
}
async fn track_code_at_path_with_changes(
    axum::extract::Path(path): axum::extract::Path<track::TrackingAtPathParam>,
    axum::extract::State(state): axum::extract::State<SharedState>,
    axum::extract::Query(query): axum::extract::Query<track::TrackingQuery>,
) -> impl IntoResponse {
    dbg!(&path);
    dbg!(&query);
    track::track_code_at_path_with_changes(state, path, query)
}

pub fn view_code_route(_st: SharedState) -> Router<SharedState> {
    let service_config = ServiceBuilder::new()
        .layer(HandleErrorLayer::new(|e: BoxError| async move {
            dbg!(e);
        }))
        .load_shed()
        .concurrency_limit(8)
        .buffer(20)
        .rate_limit(2, Duration::from_secs(5))
        // .request_body_limit(1024 * 5_000 /* ~5mb */)
        .timeout(Duration::from_secs(10))
        .layer(TraceLayer::new_for_http());
    Router::new()
        .route(
            "/view/github/:user/:name/:commit/*path",
            get(view_code).layer(service_config.clone()), // .with_state(Arc::clone(&shared_state)),
        )
        .route(
            "/view/github/:user/:name/:commit/",
            get(view_code).layer(service_config.clone()), // .with_state(Arc::clone(&shared_state)),
        )
        .route(
            "/view/:id",
            get(view_code_with_node_id).layer(service_config.clone()), // .with_state(Arc::clone(&shared_state)),
        )
}

// #[axum_macros::debug_handler]
async fn view_code(
    axum::extract::Path(path): axum::extract::Path<view::Parameters>,
    axum::extract::State(state): axum::extract::State<SharedState>,
) -> axum::response::Result<Json<view::ViewRes>> {
    dbg!(&path);
    view::view(state, path).map_err(|err| err.into())
}
async fn view_code_with_node_id(
    axum::extract::Path(id): axum::extract::Path<u64>,
    axum::extract::State(state): axum::extract::State<SharedState>,
) -> axum::response::Result<Json<view::ViewRes>> {
    view::view_with_node_id(state, id).map_err(|err| err.into())
}

pub fn fetch_code_route(_st: SharedState) -> Router<SharedState> {
    let service_config = ServiceBuilder::new()
        .layer(HandleErrorLayer::new(|e: BoxError| async move {
            dbg!(e);
        }))
        .load_shed()
        .concurrency_limit(8)
        .buffer(20)
        .rate_limit(2, Duration::from_secs(5))
        // .request_body_limit(1024 * 5_000 /* ~5mb */)
        .timeout(Duration::from_secs(10))
        .layer(TraceLayer::new_for_http());
    Router::new()
        .route(
            "/fetch/github/:user/:name/:commit/*path",
            get(fetch_code).layer(service_config.clone()), // .with_state(Arc::clone(&shared_state)),
        )
        .route(
            "/fetch/github/:user/:name/:commit/",
            get(fetch_code).layer(service_config.clone()), // .with_state(Arc::clone(&shared_state)),
        )
        .route(
            "/fetch-ids/*ids",
            get(fetch_code_with_node_ids).layer(service_config.clone()), // .with_state(Arc::clone(&shared_state)),
        )
        .route(
            "/fetch-labels/*ids",
            get(fetch_labels).layer(service_config.clone()), // .with_state(Arc::clone(&shared_state)),
        )
}
// #[axum_macros::debug_handler]
async fn fetch_code(
    axum::extract::Path(path): axum::extract::Path<fetch::Parameters>,
    axum::extract::State(state): axum::extract::State<SharedState>,
) -> axum::response::Result<fetch::FetchedNodes> {
    dbg!(&path);
    fetch::fetch(state, path).map_err(|err| err.into())
}
async fn fetch_code_with_node_ids(
    axum::extract::Path(ids): axum::extract::Path<String>,
    axum::extract::State(state): axum::extract::State<SharedState>,
) -> axum::response::Result<Timed<fetch::FetchedNodes>> {
    dbg!(&ids);
    fetch::fetch_with_node_ids(state, ids.split("/")).map_err(|err| err.into())
}
async fn fetch_labels(
    axum::extract::Path(ids): axum::extract::Path<String>,
    axum::extract::State(state): axum::extract::State<SharedState>,
) -> axum::response::Result<Timed<fetch::FetchedLabels>> {
    dbg!(&ids);
    fetch::fetch_labels(state, ids.split("/")).map_err(|err| err.into())
}

impl IntoResponse for fetch::FetchedLabels {
    fn into_response(self) -> Response {
        let resp = Json(self).into_response();
        resp
    }
}

impl IntoResponse for fetch::FetchedNodes {
    fn into_response(self) -> Response {
        dbg!();
        let to_string = serde_json::to_string(&self);
        dbg!();
        let var_name = to_string.unwrap();
        dbg!();
        let resp = var_name.into_response();
        // let resp = Json(self).into_response();
        dbg!();
        resp
    }
}

pub fn commit_metadata_route(_st: SharedState) -> Router<SharedState> {
    let service_config = ServiceBuilder::new()
        .layer(HandleErrorLayer::new(|e: BoxError| async move {
            dbg!(e);
        }))
        .load_shed()
        .concurrency_limit(8)
        .buffer(20)
        .rate_limit(2, Duration::from_secs(5))
        // .request_body_limit(1024 * 5_000 /* ~5mb */)
        .timeout(Duration::from_secs(10))
        .layer(TraceLayer::new_for_http());
    Router::new().route(
        "/commit/github/:user/:name/:version",
        get(commit_metadata).layer(service_config.clone()), // .with_state(Arc::clone(&shared_state)),
    )
}

#[axum_macros::debug_handler]
async fn commit_metadata(
    axum::extract::Path(path): axum::extract::Path<commit::Param>,
    axum::extract::State(state): axum::extract::State<SharedState>,
) -> axum::response::Result<Json<commit::Metadata>> {
    dbg!(&path);
    commit::commit_metadata(state, path).map_err(|err| err.into())
}

pub struct Timed<T> {
    pub(crate) time: f64,
    pub(crate) content: T,
}

impl<T: IntoResponse> IntoResponse for Timed<T> {
    fn into_response(self) -> Response {
        let mut resp = self.content.into_response();
        let headers = resp.headers_mut();
        // eg. Server-Timing: cache;desc="Cache Read";dur=23.2
        headers.insert(
            "Server-Timing",
            format!("db;desc=\"DB Read\";dur={}", self.time)
                .parse()
                .unwrap(),
        );
        resp
    }
}
