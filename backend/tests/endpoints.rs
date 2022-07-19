use std::net::{SocketAddr,TcpListener};

use sqlx::PgPool;
use wiremock::{MockServer, Mock, ResponseTemplate};

use test_utils::{create_db, execute_sql_from_file, init_logging};

use article_server_rs::{migrate_db,ServerArgs,start_server,
    auth::{create_jwt,Role},
    webpages::ShowWebpageResponse};

struct TestResources {
    addr: SocketAddr,
    pool: PgPool,
    jwt: String,
    admin_jwt: String,
}

// Get a random port. The TcpListener has to go out of scope to be closed and thereby release its
// bind on the address. Otherwise the server cannot bind the address since it will already be
// bound.
fn get_address() -> SocketAddr {
    // binding to port 0 gives you a random port on linux.
    let listener = TcpListener::bind("127.0.0.1:0").expect("Unable to bind tcp listener");
    listener.local_addr().expect("Unable to get socket address")
}

#[tokio::test]
async fn test_status() {
    let test_resources = start_test_server().await;
    let client = reqwest::Client::new();
    let response = client.get(format!("http://{}:{}/api/status",
            test_resources.addr.ip(), test_resources.addr.port()))
        .send()
        .await
        .expect("Error sending request to server");
    assert_eq!(response.status().is_success(), true);
}

#[tokio::test]
async fn test_fetch_webpage() {
    let test_resources = start_test_server().await;
    let client = reqwest::Client::new();
    let mock_server = MockServer::start().await;
    let html_response = "<head><script src=\"script.js\"></script><title>Title</title></head><body><p>An html document</p></body>";
    let mock_response = ResponseTemplate::new(200)
        .set_body_string(html_response);
    Mock::given(wiremock::matchers::method("GET"))
        .and(wiremock::matchers::path("fetch-page"))
        .respond_with(mock_response)
        // Expect the mock to be called exactly once.
        .expect(1)
        .mount(&mock_server)
        .await;
    let url = &format!("{}/fetch-page", mock_server.uri());
    let response = client.post(format!("http://{}:{}/api/fetch",
            test_resources.addr.ip(), test_resources.addr.port()))
        .header(reqwest::header::AUTHORIZATION, &format!("bearer {}", test_resources.jwt))
        .body(serde_json::json!({"url": url}).to_string())
        .send()
        .await
        .expect("Error sending request to server");
    assert!(response.status().is_success());
    let results = sqlx::query_as::<_, (String, String, String)>("select text, html, title from webpages where url = $1 and user_id = 1")
        .bind(url)
        .fetch_optional(&test_resources.pool).await.expect("Unable to query for fetched webpage");
    assert!(results.is_some());
    if let Some(results) = results {
        let (text, html, title) = results;
        assert_eq!(text, "<title > Title </title> <p > An html document </p>");
        assert_eq!(html, "<head><script src=\"script.js\"></script><title>Title</title></head><body><p>An html document</p></body>");
        assert_eq!(title, "Title");
    } else {
        panic!("This should not happen");
    }
}

#[tokio::test]
async fn test_fetch_webpage_html_provided() {
    let test_resources = start_test_server().await;
    let client = reqwest::Client::new();
    let mock_server = MockServer::start().await;
    let html_response = "<head><script src=\"script.js\"></script><title>Title</title></head><body><p>An html document</p></body>";
    let mock_response = ResponseTemplate::new(200)
        .set_body_string(html_response);
    Mock::given(wiremock::matchers::method("GET"))
        .and(wiremock::matchers::path("fetch-page"))
        .respond_with(mock_response)
        // Expect the mock not to be called because the client supplied the html themselves.
        .expect(0)
        .mount(&mock_server)
        .await;
    let url = &format!("{}/fetch-page", mock_server.uri());
    let response = client.post(format!("http://{}:{}/api/fetch",
            test_resources.addr.ip(), test_resources.addr.port()))
        .header(reqwest::header::AUTHORIZATION, &format!("bearer {}", test_resources.jwt))
        .body(serde_json::json!({"url": url,
            "html": "<head><script src=\"script.js\"></script><title>Self-provided title</title></head><body><p>A self-provided html document</p></body>"}).to_string())
        .send()
        .await
        .expect("Error sending request to server");
    assert_eq!(response.status().is_success(), true);
    let results = sqlx::query_as::<_, (String, String, String)>("select text, html, title from webpages where url = $1 and user_id = 1")
        .bind(url)
        .fetch_optional(&test_resources.pool).await.expect("Unable to query for fetched webpage");
    assert!(results.is_some());
    if let Some(results) = results {
        let (text, html, title) = results;
        assert_eq!(text, "<title > Self-provided title </title> <p > A self-provided html document </p>");
        assert_eq!(html, "<head><script src=\"script.js\"></script><title>Self-provided title</title></head><body><p>A self-provided html document</p></body>");
        assert_eq!(title, "Self-provided title");
    } else {
        panic!("This should not happen");
    }
}

#[tokio::test]
async fn test_fetch_webpage_error_on_fetch() {
    let test_resources = start_test_server().await;
    let client = reqwest::Client::new();
    let mock_server = MockServer::start().await;
    let mock_response = ResponseTemplate::new(500)
        .set_body_string("Some unknown error");
    Mock::given(wiremock::matchers::method("GET"))
        .and(wiremock::matchers::path("fetch-page"))
        .respond_with(mock_response)
        // Expect the mock to be called exactly once.
        .expect(1)
        .mount(&mock_server)
        .await;
    let url = &format!("{}/fetch-page", mock_server.uri());
    let response = client.post(format!("http://{}:{}/api/fetch",
            test_resources.addr.ip(), test_resources.addr.port()))
        .header(reqwest::header::AUTHORIZATION, &format!("bearer {}", test_resources.jwt))
        .body(serde_json::json!({"url": url}).to_string())
        .send()
        .await
        .expect("Error sending request to server");
    assert_eq!(response.status(), warp::http::StatusCode::NOT_FOUND);
    let error_text = format!("Unable to fetch {}/fetch-page. Got status 500 Internal Server Error: Some unknown error",
        mock_server.uri());
    assert_eq!(error_text, response.text().await.expect("Unable to get text of response"));
}

#[tokio::test]
async fn test_get_webpage() {
    let test_resources = start_test_server().await;
    execute_sql_from_file("tests/data/insert-webpage.sql", &test_resources.pool)
        .await.expect("Unable to insert webpages");
    let client = reqwest::Client::new();
    let response = client.get(format!("http://{}:{}/api/webpage/1",
            test_resources.addr.ip(), test_resources.addr.port()))
        .header(reqwest::header::AUTHORIZATION, &format!("bearer {}", test_resources.jwt))
        .send()
        .await
        .expect("Error sending request to server");
    assert_eq!(response.status().is_success(), true);
    let webpage: ShowWebpageResponse = serde_json::from_str(
        &response.text().await.expect("Unable to get text from response"))
        .expect("Unable to parse response as webpage response");
    assert_eq!(webpage, ShowWebpageResponse::new(
        "title".into(),
        Some("image_url".into()),
        "text".into(),
    ));
}

#[tokio::test]
async fn test_get_webpage_oauth2_provider_jwt() {
    let test_resources = start_test_server().await;
    let mock_server = MockServer::start().await;

    /* This test jwt has been generated with https://mkjwk.org/ and PyJWT.
     * https://pyjwt.readthedocs.io/en/stable/usage.html#encoding-decoding-tokens-with-rs256-rsa
     *
     * import jwt
     * p = {
     *     "iss": "https://test.dev",
     *     "sub": "user-1",
     *     "nbf": 1657639158,
     *     "exp": 4811324654,
     *     "iat": 1657639158,
     *     "jti": "id123456",
     *     "typ": "https://test.dev/register",
     *     "aud": "test-aud"
     * }

     * d = {
     *    "p": "-s1NXQX-p8xKK8Oo3nPBTN5brfi812g5bfWK1iwjQLKq2lEMDiGi-L_imZV8DN5Ky0HK1-LxsAeDEgUK0DRwjR_bI_60lFmKg98jnfaNwYgpy-j_sG2znTe52APThOghMi6t6mw068wN63WCoItEnj4OHwoYsN5JYyc4haCX6oU",
     *    "kty": "RSA",
     *    "q": "3_DuTm5E8LeQO7I4VizMPf4atgdVIJUoR-6Lyr-TQiXbsi6pNYVa5iDgdYtPBTrDejtc-G6JPpj7KILzUi2pxOG63x8SUP1tkpwdJ3rK6tRjQfRZoJWWNhTIxhJschKLgTTHosDqfzW9bwVHrTlK807OjV3WqPfw0U86zbcNd5E",
     *    "d": "st4YgrrtC8uMBpi1qRXR51itLc4YG3E0bg4uMN-tsZeXpOt5jSea-xYwtaQBHm7FH8I9iGLnej5MIo8kEtEZGoRKyxJmLztiefv0SPkOsK6Tuw5zGF-0W8mjgGWfpmmYriSPFQEhb1ZYrzRinHMewk82QR3785g06gK0oYoClpai_6J5L4fNY1kvXgxDKVJJCuwiThIWuxtPEKUNJU4FoJw2K3plVsnrU8o4KPkgIHy3OvgapBF1bM3E-n_8bxBnd8b0mBWPIxc1JwW8A1MlMzOfOIvS6LP3ZPVKfzpeovTymThmcmM5_8YA10yfmz_So9OlZnkHDntH3mpGEI3wQ",
     *    "e": "AQAB",
     *    "use": "sig",
     *    "kid": "oyokt7sHchlqee9Gr6vQtO7l5YFmfTuvx1QpxJfYnw0",
     *    "qi": "WF2QRsvJqgSk-d08sPNPuJwKK7LMSYPCvZQJmi-FgN4XLxHZVjEV7aZxTKrULcgcEV-POu3-UOI1XLG9H-J8zSkmeIyQUxjpTV-J99J8-CfRUFQ90Z02xi_VXMpp16KhPYGQgsk5OhrcCSPJiBbMLwFJFJFhcFK8nl13Q9vKOVU"
     *    "dp": "WzWyvvPhKvEWwFfF4DDEycnMGbbuJoCW8jBsL3uZznuruv3innkJJeHS7Pv6Q0vMc6MXu--i6duxhSokRpfrnsdJEQwebB0sTM0nzNjPsORuHuQ7qNQckD6l7bNmh11MRU3IngqALIjnPwxbVzuO1uXGiO9JD3mnwtGOsro2xWk"
     *    "alg": "RS256",
     *    "dq": "Vmf5G0wSz6qUPWRjtmRsnhLYrZmgsAS9WRvi1mUa5bAD1_mHEn6U9yyCTvhkkgj9ecFD-xtzWzLd3eDBD9lMownR99teTt-qEqKn4R7RAtDWR5GHr51oKw_T9BERxOYA6-a4jMTQ9ip_IEIySNVNZRnoOsWVWPbp9WkTsGJMEPE"
     *    "n": "22TgVEPwQm2rmTP2SsAOZf-XKSOGyhJK02k2MtgHjpuAGIvID4d0Eh3PXkVStokc-Z6tXmTfssE47by3DapXknqwvsrWeQFQ8dqDxdJgVzm9tPN7M2PFbfuKbt1iNLgTFkVQ1KegwK7Shjn9-2sVvCzeJFjuJpHz6HHLcQEPOYDG3US1Dp-ANl9QbHDPiUsIzjejduRIRv9xd0nQxEdEUHApP8q9Y5oRzSVY4h48y15Azqcs_MxwBgAPamSUYmac2vemrnfKddxqXvNrZuSfjN4ZYGC1zhUTjk1I_6jbRNlbSKGsgSbnvB2Um-dGkJaDtM311fymZXnB5tAzj9CoVQ"
     * }
     * key = jwt.algorithms.RSAPSSAlgorithm.from_jwk(d)
     * token = jwt.encode(p, key, algorithm="RS256", headers={"kid": "oyokt7sHchlqee9Gr6vQtO7l5YFmfTuvx1QpxJfYnw0"})
     */
    let jwt = "eyJ0eXAiOiJKV1QiLCJhbGciOiJSUzI1NiIsImtpZCI6Im95b2t0N3NIY2hscWVlOUdyNnZRdE83bDVZRm1mVHV2eDFRcHhKZlludzAifQ.eyJpc3MiOiJodHRwczovL3Rlc3QuZGV2Iiwic3ViIjoidXNlci0xIiwibmJmIjoxNjU3NjM5MTU4LCJleHAiOjQ4MTEzMjQ2NTQsImlhdCI6MTY1NzYzOTE1OCwianRpIjoiaWQxMjM0NTYiLCJ0eXAiOiJodHRwczovL3Rlc3QuZGV2L3JlZ2lzdGVyIiwiYXVkIjoidGVzdC1hdWQifQ.1Smq_RdkTxYZHGarJnqtokiM8WwTZyb5c1yc1Q3FxroozgXWSS75oYu7PeSygfMPYnHuw3GBMJSrc0yT9U0Lirb8EmIr49qaxzpRYg0JIuXtTaJ9Hg8rtpz08VSpSnlVtpH7EWwtoBURkR2gIHMpISa7dpWWfVratAzDWAPfpD4AJzBFgLiQ2vyPUXetEL4jd9y5qklk_m2t3FymCWBW_0Zt4WHD8SHkq08i53dpHrsPUDTTJH6QBU82MSUnk9DVjXCNsJM5vK1xkx0y48wCF8Bz7gjadZDAosw0WSnQaVQjlNpWofVTB2tYcxj52yv3xVP-H6q4SVV-reJ5kQ10jA";
    let jwk = "{\"keys\": [{\"kty\": \"RSA\", \"e\": \"AQAB\", \"use\": \"sig\", \"kid\": \"oyokt7sHchlqee9Gr6vQtO7l5YFmfTuvx1QpxJfYnw0\", \"alg\": \"RS256\", \"n\": \"22TgVEPwQm2rmTP2SsAOZf-XKSOGyhJK02k2MtgHjpuAGIvID4d0Eh3PXkVStokc-Z6tXmTfssE47by3DapXknqwvsrWeQFQ8dqDxdJgVzm9tPN7M2PFbfuKbt1iNLgTFkVQ1KegwK7Shjn9-2sVvCzeJFjuJpHz6HHLcQEPOYDG3US1Dp-ANl9QbHDPiUsIzjejduRIRv9xd0nQxEdEUHApP8q9Y5oRzSVY4h48y15Azqcs_MxwBgAPamSUYmac2vemrnfKddxqXvNrZuSfjN4ZYGC1zhUTjk1I_6jbRNlbSKGsgSbnvB2Um-dGkJaDtM311fymZXnB5tAzj9CoVQ\", \"x5c\": [\"blank\"], \"x5t\": \"blank\"}]}";
    let mock_response = ResponseTemplate::new(200)
        .set_body_raw(jwk, "application/json");
    Mock::given(wiremock::matchers::method("GET"))
        .and(wiremock::matchers::path(".well-known/jwks.json"))
        .respond_with(mock_response)
        .mount(&mock_server)
        .await;
    std::env::set_var("OAUTH2_PROVIDER_BASE_URL", mock_server.uri());
    execute_sql_from_file("tests/data/insert-webpage.sql", &test_resources.pool)
        .await.expect("Unable to insert webpages");
    let client = reqwest::Client::new();
    let response = client.get(format!("http://{}:{}/api/webpage/1?auth-type=oauth2&mode=readable",
            test_resources.addr.ip(), test_resources.addr.port()))
        .header(reqwest::header::AUTHORIZATION, &format!("bearer {}", jwt))
        .send()
        .await
        .expect("Error sending request to server");
    assert_eq!(response.status().is_success(), true);
    let webpage: ShowWebpageResponse = serde_json::from_str(
        &response.text().await.expect("Unable to get text from response"))
        .expect("Unable to parse response as webpage response");
    assert_eq!(webpage, ShowWebpageResponse::new(
        "title".into(),
        Some("image_url".into()),
        "text".into(),
    ));
}

#[tokio::test]
async fn test_get_webpage_oauth2_provider_jwt_expired_jwt() {
    let test_resources = start_test_server().await;
    let mock_server = MockServer::start().await;
    // This jwt is expired
    let jwt = "eyJ0eXAiOiJKV1QiLCJhbGciOiJSUzI1NiIsImtpZCI6Im95b2t0N3NIY2hscWVlOUdyNnZRdE83bDVZRm1mVHV2eDFRcHhKZlludzAifQ.eyJpc3MiOiJodHRwczovL3Rlc3QuZGV2Iiwic3ViIjoidXNlci0xIiwibmJmIjoxNjU3NjM5MTU4LCJleHAiOjE2NTc2MzkwNTgsImlhdCI6MTY1NzYzOTE1OCwianRpIjoiaWQxMjM0NTYiLCJ0eXAiOiJodHRwczovL3Rlc3QuZGV2L3JlZ2lzdGVyIiwiYXVkIjoidGVzdC1hdWQifQ.tpGs56V3RVwIswxsyMnDpEp1iFVkmUm44mPDUSuwkd42Zae319yQiCf4hfEd-kXpb1ic9h5-4qSXwSHGOdNtQp6b35O77Peznyavo_KJ1gbr-FI7FJKrsKT66LWgxBDJIqyXdRKmRGpBauvR5Gw48joSIq2LgDzKPcKjsrz_iSs0vtrX02h3M9fqsH2Gc_Co7miCen-q_MWrQq9F2VDIsz8_XEQ8e-ZMv-8CuD8diF03MRGzvNoltO2QzFX0Y6R0uZ8fHiHd-FFUCxf0o-2lx5pBlKAGg-Cx25pI8nDDpngcV2OIrnm0sO_fBbdls0aWYMjTb4bLsXyKAZcMFFvHcw";
    let jwk = "{\"keys\": [{\"kty\": \"RSA\", \"e\": \"AQAB\", \"use\": \"sig\", \"kid\": \"oyokt7sHchlqee9Gr6vQtO7l5YFmfTuvx1QpxJfYnw0\", \"alg\": \"RS256\", \"n\": \"22TgVEPwQm2rmTP2SsAOZf-XKSOGyhJK02k2MtgHjpuAGIvID4d0Eh3PXkVStokc-Z6tXmTfssE47by3DapXknqwvsrWeQFQ8dqDxdJgVzm9tPN7M2PFbfuKbt1iNLgTFkVQ1KegwK7Shjn9-2sVvCzeJFjuJpHz6HHLcQEPOYDG3US1Dp-ANl9QbHDPiUsIzjejduRIRv9xd0nQxEdEUHApP8q9Y5oRzSVY4h48y15Azqcs_MxwBgAPamSUYmac2vemrnfKddxqXvNrZuSfjN4ZYGC1zhUTjk1I_6jbRNlbSKGsgSbnvB2Um-dGkJaDtM311fymZXnB5tAzj9CoVQ\", \"x5c\": [\"blank\"], \"x5t\": \"blank\"}]}";
    let mock_response = ResponseTemplate::new(200)
        .set_body_raw(jwk, "application/json");
    Mock::given(wiremock::matchers::method("GET"))
        .and(wiremock::matchers::path(".well-known/jwks.json"))
        .respond_with(mock_response)
        .mount(&mock_server)
        .await;
    std::env::set_var("OAUTH2_PROVIDER_BASE_URL", mock_server.uri());
    execute_sql_from_file("tests/data/insert-webpage.sql", &test_resources.pool)
        .await.expect("Unable to insert webpages");
    let client = reqwest::Client::new();
    let response = client.get(format!("http://{}:{}/api/webpage/1?auth-type=oauth2",
            test_resources.addr.ip(), test_resources.addr.port()))
        .header(reqwest::header::AUTHORIZATION, &format!("bearer {}", jwt))
        .send()
        .await
        .expect("Error sending request to server");
    assert_eq!(response.status(), warp::http::StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_get_webpage_oauth2_provider_jwt_wrong_query_parameter() {
    let test_resources = start_test_server().await;
    let mock_server = MockServer::start().await;
    let jwt = "eyJ0eXAiOiJKV1QiLCJhbGciOiJSUzI1NiJ9.eyJpc3MiOiJodHRwczovL3Rlc3QuZGV2Iiwic3ViIjoidXNlci0xIiwibmJmIjoxNjU3NjM5MTU4LCJleHAiOjE2NTc2MzkwNTgsImlhdCI6MTY1NzYzOTE1OCwianRpIjoiaWQxMjM0NTYiLCJ0eXAiOiJodHRwczovL3Rlc3QuZGV2L3JlZ2lzdGVyIiwiYXVkIjoidGVzdC1hdWQifQ.lzzfZBxpgCWHnV4G_tEkRrQUarlrKk18K4DM3Js9-iLMoQDQx9wuOTfyOwOPGmhC-a8XqpktrDJ6gct6Qz5nbEKIb7qrkF5m071fGdvRkfttk56n_JzS9xzLzq6tGaZ369q_VOSUtRvdLgzzQ8q72HbnntCjQHTad3P2YnDFH36ORIYf39DHRR00aynioW8HhggAO5bJUSvfXxcmHjhhj1hIhEjB-pocttyOPIFJm009gApukjmiqQhmulJAs57tJDc0zK27RyiSInBEX-8Eh15OcneAxsvEMOSVOojbmSesZWwJKh8szWOAsra0x_G4BFhwmtQ1BvH53NnQxwq8GQ";
    let jwk = "{\"keys\": [{\"kty\": \"RSA\", \"e\": \"AQAB\", \"use\": \"sig\", \"kid\": \"oyokt7sHchlqee9Gr6vQtO7l5YFmfTuvx1QpxJfYnw0\", \"alg\": \"RS256\", \"n\": \"22TgVEPwQm2rmTP2SsAOZf-XKSOGyhJK02k2MtgHjpuAGIvID4d0Eh3PXkVStokc-Z6tXmTfssE47by3DapXknqwvsrWeQFQ8dqDxdJgVzm9tPN7M2PFbfuKbt1iNLgTFkVQ1KegwK7Shjn9-2sVvCzeJFjuJpHz6HHLcQEPOYDG3US1Dp-ANl9QbHDPiUsIzjejduRIRv9xd0nQxEdEUHApP8q9Y5oRzSVY4h48y15Azqcs_MxwBgAPamSUYmac2vemrnfKddxqXvNrZuSfjN4ZYGC1zhUTjk1I_6jbRNlbSKGsgSbnvB2Um-dGkJaDtM311fymZXnB5tAzj9CoVQ\", \"x5c\": [\"blank\"], \"x5t\": \"blank\"}]}";
    let mock_response = ResponseTemplate::new(200)
        .set_body_raw(jwk, "application/json");
    Mock::given(wiremock::matchers::method("GET"))
        .and(wiremock::matchers::path(".well-known/jwks.json"))
        .respond_with(mock_response)
        .mount(&mock_server)
        .await;
    std::env::set_var("OAUTH2_PROVIDER_BASE_URL", mock_server.uri());
    execute_sql_from_file("tests/data/insert-webpage.sql", &test_resources.pool)
        .await.expect("Unable to insert webpages");
    let client = reqwest::Client::new();
    // The query parameters here are wrong
    let response = client.get(format!("http://{}:{}/api/webpage/1?auth-type=oauth3",
            test_resources.addr.ip(), test_resources.addr.port()))
        .header(reqwest::header::AUTHORIZATION, &format!("bearer {}", jwt))
        .send()
        .await
        .expect("Error sending request to server");
    assert_eq!(response.status(), warp::http::StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_register_oauth2_provider_jwt_insufficient_role() {
    let test_resources = start_test_server().await;
    let mock_server = MockServer::start().await;
    // The sub of this jwt user 1 who has insufficient privileges to access the ressource at
    // /api/register
    let jwt = "eyJ0eXAiOiJKV1QiLCJhbGciOiJSUzI1NiIsImtpZCI6Im95b2t0N3NIY2hscWVlOUdyNnZRdE83bDVZRm1mVHV2eDFRcHhKZlludzAifQ.eyJpc3MiOiJodHRwczovL3Rlc3QuZGV2Iiwic3ViIjoidXNlci0xIiwibmJmIjoxNjU3NjM5MTU4LCJleHAiOjQ4MTEzMjQ2NTQsImlhdCI6MTY1NzYzOTE1OCwianRpIjoiaWQxMjM0NTYiLCJ0eXAiOiJodHRwczovL3Rlc3QuZGV2L3JlZ2lzdGVyIiwiYXVkIjoidGVzdC1hdWQifQ.1Smq_RdkTxYZHGarJnqtokiM8WwTZyb5c1yc1Q3FxroozgXWSS75oYu7PeSygfMPYnHuw3GBMJSrc0yT9U0Lirb8EmIr49qaxzpRYg0JIuXtTaJ9Hg8rtpz08VSpSnlVtpH7EWwtoBURkR2gIHMpISa7dpWWfVratAzDWAPfpD4AJzBFgLiQ2vyPUXetEL4jd9y5qklk_m2t3FymCWBW_0Zt4WHD8SHkq08i53dpHrsPUDTTJH6QBU82MSUnk9DVjXCNsJM5vK1xkx0y48wCF8Bz7gjadZDAosw0WSnQaVQjlNpWofVTB2tYcxj52yv3xVP-H6q4SVV-reJ5kQ10jA";
    let jwk = "{\"keys\": [{\"kty\": \"RSA\", \"e\": \"AQAB\", \"use\": \"sig\", \"kid\": \"oyokt7sHchlqee9Gr6vQtO7l5YFmfTuvx1QpxJfYnw0\", \"alg\": \"RS256\", \"n\": \"22TgVEPwQm2rmTP2SsAOZf-XKSOGyhJK02k2MtgHjpuAGIvID4d0Eh3PXkVStokc-Z6tXmTfssE47by3DapXknqwvsrWeQFQ8dqDxdJgVzm9tPN7M2PFbfuKbt1iNLgTFkVQ1KegwK7Shjn9-2sVvCzeJFjuJpHz6HHLcQEPOYDG3US1Dp-ANl9QbHDPiUsIzjejduRIRv9xd0nQxEdEUHApP8q9Y5oRzSVY4h48y15Azqcs_MxwBgAPamSUYmac2vemrnfKddxqXvNrZuSfjN4ZYGC1zhUTjk1I_6jbRNlbSKGsgSbnvB2Um-dGkJaDtM311fymZXnB5tAzj9CoVQ\", \"x5c\": [\"blank\"], \"x5t\": \"blank\"}]}";
    let mock_response = ResponseTemplate::new(200)
        .set_body_raw(jwk, "application/json");
    Mock::given(wiremock::matchers::method("GET"))
        .and(wiremock::matchers::path(".well-known/jwks.json"))
        .respond_with(mock_response)
        .mount(&mock_server)
        .await;
    std::env::set_var("OAUTH2_PROVIDER_BASE_URL", mock_server.uri());
    let client = reqwest::Client::new();
    let response = client.post(format!("http://{}:{}/api/register?auth-type=oauth2",
            test_resources.addr.ip(), test_resources.addr.port()))
        .header(reqwest::header::AUTHORIZATION, &format!("bearer {}", jwt))
        .body(serde_json::json!({"username": "new-user", "password": "Password123"}).to_string())
        .send()
        .await
        .expect("Error sending request to server");
    assert_eq!(response.status(), warp::http::StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_delete_webpage() {
    let test_resources = start_test_server().await;
    execute_sql_from_file("tests/data/insert-webpage.sql", &test_resources.pool).await.expect("Unable to insert webpages");
    let results = sqlx::query_as::<_, (String, String)>("SELECT url, title FROM webpages WHERE id = 1")
        .fetch_optional(&test_resources.pool).await.expect("Unable to query for webpage prior to deleting");
    assert_eq!(results.is_some(), true);
    let client = reqwest::Client::new();
    let response = client.delete(format!("http://{}:{}/api/webpage/1",
            test_resources.addr.ip(), test_resources.addr.port()))
        .header(reqwest::header::AUTHORIZATION, &format!("bearer {}", test_resources.jwt))
        .send()
        .await
        .expect("Error sending request to server");
    assert_eq!(response.status().is_success(), true);
    let results = sqlx::query_as::<_, (String, String)>("SELECT url, title FROM webpages WHERE id = 1")
        .fetch_optional(&test_resources.pool).await.expect("Unable to query for webpage after deleting");
    assert_eq!(results.is_some(), false);
}

#[tokio::test]
async fn test_register_user() {
    let test_resources = start_test_server().await;
    let users_pre = sqlx::query_as::<_, (i64,)>("select count(id) from users")
        .fetch_one(&test_resources.pool).await.expect("Unable to get user count before registering");
    assert_eq!(users_pre.0, 2);
    let client = reqwest::Client::new();
    let response = client.post(format!("http://{}:{}/api/register",
            test_resources.addr.ip(), test_resources.addr.port()))
        .body(serde_json::json!({"username": "new-user", "password": "Password123"}).to_string())
        .header(reqwest::header::AUTHORIZATION, &format!("bearer {}", test_resources.admin_jwt))
        .send()
        .await
        .expect("Error sending request to server");
    assert_eq!(response.status().is_success(), true);
    let users_post = sqlx::query_as::<_, (String, Vec<Role>)>("select username, roles from users where username = 'new-user'")
        .fetch_one(&test_resources.pool).await.expect("Unable to get user count before registering");
    assert_eq!(users_post.0, "new-user");
    assert_eq!(users_post.1.len(), 1);
    assert_eq!(users_post.1[0], Role::User);
}

#[tokio::test]
async fn test_register_user_not_allowed_for_non_admins() {
    let test_resources = start_test_server().await;
    let client = reqwest::Client::new();
    let response = client.post(format!("http://{}:{}/api/register",
            test_resources.addr.ip(), test_resources.addr.port()))
        .body(serde_json::json!({"username": "new-user", "password": "Password123"}).to_string())
        .header(reqwest::header::AUTHORIZATION, &format!("bearer {}", test_resources.jwt))
        .send()
        .await
        .expect("Error sending request to server");
    assert_eq!(response.status(), warp::http::StatusCode::UNAUTHORIZED);
    let new_user = sqlx::query_as::<_, (String, Vec<Role>)>("select username, roles from users where username = 'new-user'")
        .fetch_optional(&test_resources.pool).await.expect("Unable to query for new user");
    assert!(new_user.is_none());
}

#[tokio::test]
async fn test_associate_app_to_user() {
    let test_resources = start_test_server().await;
    let apps_pre = sqlx::query_as::<_, (i64,)>("select count(id) from connected_apps")
        .fetch_one(&test_resources.pool).await.expect("Unable to get app count before registering");
    assert_eq!(apps_pre.0, 1);
    let client = reqwest::Client::new();
    let now = chrono::offset::Utc::now().format("%Y-%m-%dT%H:%M:%S");
    let response = client.post(format!("http://{}:{}/api/associate-app-to-user",
            test_resources.addr.ip(), test_resources.addr.port()))
        .body(serde_json::json!({"sub": "sub", "client_id": &format!("client_id_{}", now),
            "app_host": &format!("app_host_{}", now)}).to_string())
        .header(reqwest::header::AUTHORIZATION, &format!("bearer {}", test_resources.jwt))
        .send()
        .await
        .expect("Error sending request to server");
    assert_eq!(response.status(), warp::http::StatusCode::CREATED);
    let apps_post = sqlx::query_as::<_, (i64,String,String,String)>(
            "select user_id, sub, client_id, app_host from connected_apps where user_id = 1 order by id desc limit 1")
        .fetch_optional(&test_resources.pool).await.expect(
            "Unable to get app count before registering");
    assert_eq!(apps_post.is_some(), true);
    if let Some(apps_post) = apps_post {
        let (user_id, sub, client_id, app_host) = apps_post;
        assert_eq!(user_id, 1);
        assert_eq!(sub, "sub");
        assert_eq!(client_id, format!("client_id_{}", now));
        assert_eq!(app_host, format!("app_host_{}", now));
    }
}

#[tokio::test]
async fn test_associate_app_to_user_existing_app() {
    let test_resources = start_test_server().await;
    let client = reqwest::Client::new();

    sqlx::query("insert into connected_apps(user_id, sub, client_id, app_host) values (1, 'sub', 'client_id', 'app_host')")
        .execute(&test_resources.pool).await
        .expect("Unable to insert app into database");

    let response = client.post(format!("http://{}:{}/api/associate-app-to-user",
            test_resources.addr.ip(), test_resources.addr.port()))
        .body(serde_json::json!({"sub": "sub", "client_id": "client_id",
            "app_host": "app_host"}).to_string())
        .header(reqwest::header::AUTHORIZATION, &format!("bearer {}", test_resources.jwt))
        .send()
        .await
        .expect("Error sending request to server");
    assert_eq!(response.status(), warp::http::StatusCode::CREATED);
    let apps_post = sqlx::query_as::<_, (i64,String,String,String)>(
            "select user_id, sub, client_id, app_host from connected_apps where user_id = 1 order by id desc")
        .fetch_one(&test_resources.pool).await.expect(
            "Unable to get app count before registering");
    let (user_id, sub, client_id, app_host) = apps_post;
    assert_eq!(user_id, 1);
    assert_eq!(sub, "sub");
    assert_eq!(client_id, "client_id");
    // The app host must be something other than "app_host" because that values already exists.
    assert!(app_host != "app_host");
}

#[tokio::test]
async fn test_associate_app_to_user_malformed_json() {
    let test_resources = start_test_server().await;
    let client = reqwest::Client::new();
    let response = client.post(format!("http://{}:{}/api/associate-app-to-user",
            test_resources.addr.ip(), test_resources.addr.port()))
        .body(serde_json::json!({"hur": 123}).to_string())
        .header(reqwest::header::AUTHORIZATION, &format!("bearer {}", test_resources.jwt))
        .send()
        .await
        .expect("Error sending request to server");
    assert_eq!(response.status(), warp::http::StatusCode::BAD_REQUEST);
}

async fn start_test_server() -> TestResources {
    init_logging();
    let addr = get_address();
    let pool = create_db().await;
    let cloned_pool = pool.clone();
    migrate_db(&pool).await.expect("Unable to migrate database");
    execute_sql_from_file("tests/data/auth.sql", &pool).await.expect("Unable to insert auth data");
    let (user, secret) = sqlx::query_as::<_, (String, String)>("SELECT users.username, jwt_secrets.secret FROM jwt_secrets JOIN users on users.id = jwt_secrets.user_id where users.id = 1").fetch_one(&pool).await.expect("Unable to fetch jwt secret");
    let jwt = create_jwt(&user, secret.as_ref(), 60 * 60 * 24).expect("Error creating jwt");
    let (admin_user, admin_secret) = sqlx::query_as::<_, (String, String)>("SELECT users.username, jwt_secrets.secret FROM jwt_secrets JOIN users on users.id = jwt_secrets.user_id where users.id = 2").fetch_one(&pool).await.expect("Unable to fetch admin jwt secret");
    let admin_jwt = create_jwt(&admin_user, admin_secret.as_ref(), 60 * 60 * 24).expect("Error creating admin jwt");
    let _ = tokio::spawn(async move {
        // Starting a task with a server started by warp::Server::run is possibly impossible to do
        // at the moment. I get the error
        // 28 |         rt.spawn(setup_server(addr));
        //    |            ^^^^^ implementation of `warp::reply::Reply` is not general enough
        //    |
        //    = note: `&'0 str` must implement `warp::reply::Reply`, for any lifetime `'0`...
        //    = note: ...but `warp::reply::Reply` is actually implemented for the type `&'static str`
        // I could get it to work by using Server::bind instead but that then creates an
        // intermediate Future and that's why I have the double await here.
        setup_server(addr, cloned_pool).await.await;
    });
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    TestResources {
        addr,
        pool,
        jwt,
        admin_jwt,
    }
}

async fn setup_server(addr: SocketAddr, pool: PgPool) -> impl core::future::Future {
    let server_args = ServerArgs {
        pool,
        addr,
    };
    start_server(server_args)
}
