
use std::{sync::Arc, borrow::BorrowMut};

use axum::{
    Json,
    extract::{State, Path},
    routing::{
        Router,
        get, post
    }
};
use hyper::StatusCode;
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::{models::{
    user::{User, CreateUserRequest, LoginUserRequest},
    detailed_response::DetailedResponse,
    state::YaddakState,
    errors::YaddakErrorKind
}, traits::repo::Repo};

async fn register(
    State(state): State<Arc<Mutex<YaddakState>>>,
    Json(payload): Json<CreateUserRequest>,
) -> (StatusCode, Json<DetailedResponse<User>>) {
    let client = &state.lock().await.db;
    
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

async fn login(
    State(state): State<Arc<Mutex<YaddakState>>>,
    Json(payload): Json<LoginUserRequest>
) -> (StatusCode, Json<DetailedResponse<User>>) {
    let client = &state.lock().await.db;
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

async fn get_user(
    State(state): State<Arc<Mutex<YaddakState>>>,
    Path(id): Path<Uuid>
) -> (StatusCode, Json<DetailedResponse<User>>) {
    let client = &state.lock().await.db;
    match User::get(client.clone(), id).await {
        Ok(user) => (StatusCode::OK, Json(DetailedResponse::absorb_data(user))),
        Err(err) => (StatusCode::OK, Json(DetailedResponse::absorb_error(err)))
    }    
}

async fn get_all(
    State(state): State<Arc<Mutex<YaddakState>>>,
) -> (StatusCode, Json<DetailedResponse<Vec<User>>>) {
    let client = &state.lock().await.db;
    match User::get_all(client.clone()).await {
        Ok(user) => (StatusCode::OK, Json(DetailedResponse::absorb_data(user))),
        Err(err) => (StatusCode::OK, Json(DetailedResponse::absorb_error(err)))
    }    
}

async fn update(
    State(state): State<Arc<Mutex<YaddakState>>>,
    Path(id): Path<Uuid>,
    Json(payload): Json<User>
) -> (StatusCode, Json<DetailedResponse<User>>) {
    let client = &state.lock().await.db;
    match User::put(client.clone(), id, &payload).await {
        Ok(()) => (StatusCode::OK, Json(DetailedResponse::absorb_data(payload))),
        Err(err) => (StatusCode::NOT_MODIFIED, Json(DetailedResponse::absorb_error(err)))
    }
}

async fn remove(
    State(state): State<Arc<Mutex<YaddakState>>>,
    Path(id): Path<Uuid>,
) -> (StatusCode, Json<DetailedResponse<Uuid>>) {
    let client = &state.lock().await.db;
    match User::delete(client.clone(), id).await {
        Ok(()) => (StatusCode::OK, Json(DetailedResponse::absorb_data(id))),
        Err(err) => (StatusCode::NOT_MODIFIED, Json(DetailedResponse::absorb_error(err)))
    }
}

pub fn user_controller(state: Arc<Mutex<YaddakState>>) -> Router {
    Router::new()
        .route("/", get(get_all).post(register))
        .route("/:id", get(get_user).put(update).delete(remove))
        .route("/login", post(login))
        .with_state(state)
}
