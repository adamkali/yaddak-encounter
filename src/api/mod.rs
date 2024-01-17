use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

pub mod user_controller;

use crate::models::user;
use crate::models::detailed_response;
use crate::models::errors;

#[derive(OpenApi)]
#[openapi(
    paths(
       user_controller::login,
       user_controller::remove,
       user_controller::update,
       user_controller::get_user,
       user_controller::get_all,
       user_controller::register,
    ),
    components(
        schemas(
            user::User,
            user::CreateUserRequest,
            user::LoginUserRequest,
            errors::YaddakError,
            errors::YaddakErrorKind,
            crate::api::detailed_response::UserDetailedResponse,
            crate::api::detailed_response::UserListDetailedResponse,
            crate::api::detailed_response::UuidDetailedResponse,
        )
    ),
    tags(
        (name = "Yaddak Encounter API", description="Yet Another Dungeons and Dragons All in one Kit")
    )
)]
pub struct ApiDocs;

pub fn api_docs() -> SwaggerUi {
    SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDocs::openapi())
}
