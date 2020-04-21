use actix_web::{
    http,
    web::{Data, Json, Query},
    HttpResponse,
};
use serde_json::json;

use crate::server::error::{ErrorCode, MatrixError, ResultExt};
use crate::{db::Store, models::registration as model};

/// Checks to see if a username is available, and valid, for the server.
///
/// The server should check to ensure that, at the time of the request, the username
/// requested is available for use. This includes verifying that an application service
/// has not claimed the username and that the username fits the server's desired
/// requirements (for example, a server could dictate that it does not permit usernames
/// with underscores).
///
/// Matrix clients may wish to use this API prior to attempting registration, however
/// the clients must also be aware that using this API does not normally reserve the username.
/// This can mean that the username becomes unavailable between checking its availability
/// and attempting to register it.
pub async fn get_available<T: Store>(
    params: Query<model::AvailableParams>,
    storage: Data<T>,
) -> Result<HttpResponse, MatrixError> {
    // TODO: !!!Validate Username:
    // M_INVALID_USERNAME : The desired username is not a valid user name.
    // TODO: M_EXCLUSIVE : The desired username is in the exclusive namespace claimed by an application service.

    if !model::is_username_valid(&params.username) {
        Err(MatrixError::new(
            http::StatusCode::BAD_REQUEST,
            ErrorCode::INVALID_USERNAME,
            "The desired username is not a valid user name.",
        ))?
    }

    let exists = storage
        .check_username_exists(&params.username)
        .await
        .unknown()?;

    if exists {
        Err(MatrixError::new(
            http::StatusCode::BAD_REQUEST,
            ErrorCode::USER_IN_USE,
            "Desired user ID is already taken.",
        ))?
    } else {
        Ok(HttpResponse::Ok().json(json!({"avaiable": true})))
    }
}

/// This API endpoint uses the User-Interactive Authentication API_, except in the
/// cases where a guest account is being registered.
///
/// Register for an account on this homeserver.
///
/// There are two kinds of user account:
///
///     user accounts. These accounts may use the full API described in this
/// specification.
///
///     guest accounts. These accounts may have limited permissions and may not be
/// supported by all servers.
///
/// If registration is successful, this endpoint will issue an access token the client
/// can use to authorize itself in subsequent requests.
///
/// If the client does not supply a device_id, the server must auto-generate one.
///
/// The server SHOULD register an account with a User ID based on the username provided,
/// if any. Note that the grammar of Matrix User ID localparts is restricted, so the
/// server MUST either map the provided username onto a user_id in a logical manner, or
/// reject username\s which do not comply to the grammar, with M_INVALID_USERNAME.
///
/// Matrix clients MUST NOT assume that localpart of the registered user_id matches the
/// provided username.
///
/// The returned access token must be associated with the device_id supplied by the client
///  or generated by the server. The server may invalidate any access token previously
/// associated with that device. See Relationship between access tokens and devices_.
///
/// When registering a guest account, all parameters in the request body with the exception
/// of initial_device_display_name MUST BE ignored by the server. The server MUST pick a
/// device_id for the account regardless of input.
///
/// Any user ID returned by this API must conform to the grammar given in the Matrix specification_.
pub async fn post_register<T: Store>(
    params: Query<model::RequestParams>,
    mut req: Json<model::Request>,
    storage: Data<T>,
) -> Result<HttpResponse, MatrixError> {
    req.kind = params.kind.clone();
    println!("{}", storage.get_type());

    unimplemented!()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::mock::MockStore;
    use actix_web::{http, test, web, App};

    #[actix_rt::test]
    async fn test_get_available_username_taken() {
        crate::init_config_from_file(".env-test");

        let mut test_db = MockStore::new();
        test_db.check_username_exists_resp = Some(Ok(true));

        let mut app = test::init_service(
            App::new()
                .data(test_db)
                .route("/", web::get().to(get_available::<MockStore>)),
        )
        .await;
        let req = test::TestRequest::get()
            .uri("/?username=taken")
            .header(http::header::CONTENT_TYPE, "application/json")
            .to_request();
        let resp = test::call_service(&mut app, req).await;

        assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST);
    }

    #[actix_rt::test]
    async fn test_get_available_username_available() {
        crate::init_config_from_file(".env-test");

        let mut test_db = MockStore::new();
        test_db.check_username_exists_resp = Some(Ok(false));

        let mut app = test::init_service(
            App::new()
                .data(test_db)
                .route("/", web::get().to(get_available::<MockStore>)),
        )
        .await;
        let req = test::TestRequest::get()
            .uri("/?username=taken_nottaken")
            .header(http::header::CONTENT_TYPE, "application/json")
            .to_request();
        let resp = test::call_service(&mut app, req).await;

        assert!(resp.status().is_success());
    }

    #[actix_rt::test]
    async fn test_get_available_username_invalid() {
        crate::init_config_from_file(".env-test");

        let mut test_db = MockStore::new();
        test_db.check_username_exists_resp = Some(Ok(false));

        let mut app = test::init_service(
            App::new()
                .data(test_db)
                .route("/", web::get().to(get_available::<MockStore>)),
        )
        .await;
        let req = test::TestRequest::get()
            .uri("/?username=t@ken")
            .header(http::header::CONTENT_TYPE, "application/json")
            .to_request();
        let resp = test::call_service(&mut app, req).await;

        assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST);
    }
}
