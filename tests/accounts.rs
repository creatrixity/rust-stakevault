pub mod utils;

use sqlx::{ PgConnection, Connection };
use rust_stakevault::configuration::get_configuration;

#[actix_rt::test]
async fn account_creation_passes_when_valid_fields_passed() {
    let address = utils::spawn_app();
    let configuration = get_configuration().expect("Failed to read configuration");
    let connection_string = configuration.database.connection_string();
    let mut connection = PgConnection::connect(&connection_string)
        .await
        .expect("Failed to connect to PostgreSQL");
    let client = reqwest::Client::new();
    let body:&str = "email=creatrixity%40gmail.com&username=creatrixity&password=secret";

    let response = client
        .post(&format!("{}/account/create", &address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status().as_u16(), 200);

    let account = sqlx::query!("SELECT username, email from accounts")
        .fetch_one(&mut connection)
        .await
        .expect("Failed to fetch an account");

    assert_eq!(account.username, "john.doe");
    assert_eq!(account.email, "john.doe@example.com");
}

#[actix_rt::test]
async fn account_creation_fails_when_invalid_fields_passed() {
    let address = utils::spawn_app();
    let client = reqwest::Client::new();
    let test_cases = vec![
        ("email=creatrixity%40gmail.com", "only email was passed"),
        ("username=creatrixity", "only username was passed"),
        ("", "no data was passed"),
    ];


    for (value, reason) in test_cases {
        let response = client
        .post(&format!("{}/account/create", &address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(value)
        .send()
        .await
        .expect("Failed to execute request");

        assert_eq!(400, response.status().as_u16(), "Failed with a 400 Bad Request because {}", reason);
    }
}
