pub mod utils;

#[actix_rt::test]
async fn account_creation_passes_when_valid_fields_passed() {
    let app = utils::spawn_app().await;
    let client = reqwest::Client::new();
    let body: &str = "email=creatrixity%40gmail.com&username=creatrixity&password=secret";

    let response = client
        .post(&format!("{}/account/create", app.address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status().as_u16(), 200);

    let account = sqlx::query!("SELECT username, email from accounts")
        .fetch_one(&app.connection_pool)
        .await
        .expect("Failed to fetch an account");

    assert_eq!(account.username, "creatrixity");
    assert_eq!(account.email, "creatrixity@gmail.com");
}

#[actix_rt::test]
async fn account_creation_fails_when_invalid_fields_passed() {
    let app = utils::spawn_app().await;
    let client = reqwest::Client::new();
    let test_cases = vec![
        ("email=creatrixity%40gmail.com", "only email was passed"),
        ("username=creatrixity", "only username was passed"),
        ("", "no data was passed"),
    ];

    for (value, reason) in test_cases {
        let response = client
            .post(&format!("{}/account/create", &app.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(value)
            .send()
            .await
            .expect("Failed to execute request");

        assert_eq!(
            400,
            response.status().as_u16(),
            "Failed with a 400 Bad Request because {}",
            reason
        );
    }
}

#[actix_rt::test]
async fn account_creation_passes_when_fields_are_present_but_empty() {
    let app = utils::spawn_app().await;
    let client = reqwest::Client::new();

    let test_cases = vec![
        ("username=kay&email=", "empty email"),
        ("username=kay&email=yo-not-an-email", "invalid email"),
    ];

    for (value, reason) in test_cases {
        let response = client
            .post(&format!("{}/account/create", &app.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(value)
            .send()
            .await
            .expect("Failed to execute request");

        assert_eq!(
            200,
            response.status().as_u16(),
            "The API Request did not return a 200 OK when payload was {}",
            reason
        );
    
    }
}
