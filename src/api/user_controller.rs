use std::sync::Arc;

use axum::{
    Json,
    extract::{State, Path},
    routing::{
        Router,
        get, post
    }
};
use hyper::{StatusCode, HeaderMap};
use uuid::Uuid;

use crate::{models::{
    user::{User, CreateUserRequest, LoginUserRequest},
    detailed_response::{
        DetailedResponse,
        UserDetailedResponse,
        UserListDetailedResponse,
        UuidDetailedResponse
    },
    state::YaddakState,
    errors::YaddakErrorKind
}, traits::repo::Repo, utilities::headers::auth_handler};


#[utoipa::path(
    post,
    path = "/user",
    request_body=CreateUserRequest,
    responses(
        (status = 200, description = "Created", body = UserDetailedResponse),
        (status = StatusCode::BAD_REQUEST, body = UserDetailedResponse),
        (status = StatusCode::INTERNAL_SERVER_ERROR, body = UserDetailedResponse)
    )
)]
pub async fn register(
    State(state): State<Arc<YaddakState>>,
    Json(payload): Json<CreateUserRequest>,
) -> (StatusCode, Json<UserDetailedResponse>) {
    let client = &state.db;
    
    match User::create(
        client.clone(),
        payload.user_name,
        payload.user_email,
        payload.user_pass,
    ).await {
        Ok(user) => {
            (StatusCode::OK, Json(DetailedResponse::absorb_data(user)))
        }
        Err(e) => {
            match e.kind {
                YaddakErrorKind::AuthError => 
                    (StatusCode::BAD_REQUEST, Json(DetailedResponse::absorb_error(e))),
                _ => 
                    (StatusCode::INTERNAL_SERVER_ERROR, Json(DetailedResponse::absorb_error(e)))

            }
        }
    }
}

#[utoipa::path(
    post,
    path = "/user/login",
    request_body=LoginUserRequest,
    responses(
        (status = 200, description = "Created", body = UserDetailedResponse),
        (status = StatusCode::NOT_FOUND, body = UserDetailedResponse),
        (status = StatusCode::UNAUTHORIZED, body = UserDetailedResponse)
    )
)]
pub(super) async fn login(
    State(state): State<Arc<YaddakState>>,
    Json(payload): Json<LoginUserRequest>
) -> (StatusCode, Json<UserDetailedResponse>) {
    let client = &state.db;
    match User::get_user_by_name(
        client.clone(),
        payload.user_name.clone()
    ).await {
        Ok(user) => {
            match user.authenticate(
                payload.user_name,
                payload.user_pass
            ) {
                Ok(()) => {
                    (StatusCode::OK, Json(DetailedResponse::absorb_data(user)))
                },
                Err(err) => {
                    (StatusCode::UNAUTHORIZED, Json(DetailedResponse::absorb_error(err)))
                }
            }
        },
        Err(err) => {
            (StatusCode::NOT_FOUND, Json(DetailedResponse::absorb_error(err)))
        }
    }
}

#[utoipa::path(
    get,
    path = "/auth/user/{id}",
    responses(
        (status = 200, description = "Created", body = UserDetailedResponse),
        (status = StatusCode::UNAUTHORIZED, body = UserDetailedResponse),
        (status = StatusCode::FORBIDDEN, body = UserDetailedResponse),
        (status = StatusCode::INTERNAL_SERVER_ERROR, body = UserDetailedResponse)
    ),
    params(
        ("id"=Uuid, Path, description = "ID of the user")
    )
)]
pub(super) async fn get_user(
    State(state): State<Arc<YaddakState>>,
    headers: HeaderMap,
    Path(id): Path<Uuid>
) -> (StatusCode, Json<UserDetailedResponse>) {
    let client = &state.db;
    match auth_handler(headers) {
        Ok(ah) => {
           if let Err(e) = User::check_auth(client.clone(), ah).await {
                return (StatusCode::UNAUTHORIZED, Json(DetailedResponse::absorb_error(e)));
           }
        },
        Err(err) => return (StatusCode::FORBIDDEN, Json(DetailedResponse::absorb_error(err)))
    }
    match User::get(client.clone(), id).await {
        Ok(user) => (StatusCode::OK, Json(DetailedResponse::absorb_data(user))),
        Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, Json(DetailedResponse::absorb_error(err)))
    }    
}

#[utoipa::path(
    get,
    path = "/auth/user",
    responses(
        (status = 200, description = "Created", body = UserDetailedResponse),
        (status = StatusCode::BAD_REQUEST, body = UserDetailedResponse),
        (status = StatusCode::INTERNAL_SERVER_ERROR, body = UserDetailedResponse)
    ),
)]
pub(super) async fn get_all(
    State(state): State<Arc<YaddakState>>,
    headers: HeaderMap,
) -> (StatusCode, Json<UserListDetailedResponse>) {
    let client = &state.db;
    match auth_handler(headers) {
        Ok(ah) => {
           if let Err(e) = User::check_auth(client.clone(), ah).await {
                return (StatusCode::UNAUTHORIZED, Json(DetailedResponse::absorb_error(e)));
           }
        },
        Err(err) => return (StatusCode::FORBIDDEN, Json(DetailedResponse::absorb_error(err)))
    }

    match User::get_all(client.clone()).await {
        Ok(user) => (StatusCode::OK, Json(DetailedResponse::absorb_data(user))),
        Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, Json(DetailedResponse::absorb_error(err)))
    }    
}

#[utoipa::path(
    put,
    path = "/auth/user/{id}",
    request_body = User,
    responses(
        (status = 200, description = "Created", body = UserDetailedResponse),
        (status = StatusCode::UNAUTHORIZED, body = UserDetailedResponse),
        (status = StatusCode::FORBIDDEN, body = UserDetailedResponse),
        (status = StatusCode::INTERNAL_SERVER_ERROR, body = UserDetailedResponse)
    ),
    params(
        ("id"=Uuid, Path, description = "ID of the user")
    )
)]
pub(super) async fn update(
    State(state): State<Arc<YaddakState>>,
    Path(id): Path<Uuid>,
    headers: HeaderMap,
    Json(payload): Json<User>
) -> (StatusCode, Json<UserDetailedResponse>) {
    let client = &state.db;
    match auth_handler(headers) {
        Ok(ah) => {
           if let Err(e) = User::check_auth(client.clone(), ah).await {
                return (StatusCode::UNAUTHORIZED, Json(DetailedResponse::absorb_error(e)));
           }
        },
        Err(err) => return (StatusCode::FORBIDDEN, Json(DetailedResponse::absorb_error(err)))
    }
    match User::put(client.clone(), id, &payload).await {
        Ok(()) => (StatusCode::OK, Json(DetailedResponse::absorb_data(payload))),
        Err(err) => (StatusCode::NOT_MODIFIED, Json(DetailedResponse::absorb_error(err)))
    }
}

#[utoipa::path(
    put,
    path = "/auth/user/{id}",
    request_body = User,
    responses(
        (status = 200, description = "Created", body = UserDetailedResponse),
        (status = StatusCode::UNAUTHORIZED, body = UserDetailedResponse),
        (status = StatusCode::FORBIDDEN, body = UserDetailedResponse),
        (status = StatusCode::INTERNAL_SERVER_ERROR, body = UserDetailedResponse)
    ),
    params(
        ("id"=Uuid, Path, description = "ID of the user")
    )
)]
pub(super) async fn remove(
    State(state): State<Arc<YaddakState>>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> (StatusCode, Json<UuidDetailedResponse>) {
    let client = &state.db;
    match auth_handler(headers) {
        Ok(ah) => {
           if let Err(e) = User::check_auth(client.clone(), ah).await {
                return (StatusCode::UNAUTHORIZED, Json(DetailedResponse::absorb_error(e)));
           }
        },
        Err(err) => return (StatusCode::FORBIDDEN, Json(DetailedResponse::absorb_error(err)))
    }
    match User::delete(client.clone(), id).await {
        Ok(()) => (StatusCode::OK, Json(DetailedResponse::absorb_data(id))),
        Err(err) => (StatusCode::NOT_MODIFIED, Json(DetailedResponse::absorb_error(err)))
    }
}

pub fn user_controller(state: Arc<YaddakState>) -> Router {
    Router::new()
        .route("/", post(register))
        .route("/login", post(login))
        .with_state(state)
}

pub fn user_auth_controller(state: Arc<YaddakState>) -> Router {
    Router::new()
        .route("/", get(get_all))
        .route("/:id", get(get_user).put(update).delete(remove))
        .with_state(state)
}

