use wiremock::{Mock, ResponseTemplate};
use wiremock::matchers::{ path, method };

use crate::utils;

#[actix_rt::test]
async fn subscriber_creation_returns_a_200_for_valid_form_data() {
    let app = utils::spawn_app().await;
    let body = "email=creatrixity%40gmail.com&name=caleb";

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&app.email_server)
        .await;

    let response = app.post_subscriptions(body.into()).await;
    assert_eq!(response.status().as_u16(), 200);
}

#[actix_rt::test]
async fn subscriber_creation_persists_new_subscriber() {
    let app = utils::spawn_app().await;
    let body = "email=creatrixity%40gmail.com&name=caleb";

    app.post_subscriptions(body.into()).await;

    let subscriber = sqlx::query!("SELECT name, email, status from subscriptions")
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch a subscriber");

    assert_eq!(subscriber.name, "caleb");
    assert_eq!(subscriber.email, "creatrixity@gmail.com");
    assert_eq!(subscriber.status, "pending_confirmation");
}

#[actix_rt::test]
pub async fn confirmations_without_tokens_fail() {
    // Arrange
    let app = utils::spawn_app().await;

    // Act
    let response = reqwest::get(format!("{}/subscriptions/confirm", &app.address))
        .await
        .unwrap();

    // Assert
    assert_eq!(response.status().as_u16(), 400);
}

#[actix_rt::test]
async fn subscriber_creation_fails_when_invalid_fields_passed() {
    let app = utils::spawn_app().await;
    let test_cases = vec![
        ("email=creatrixity%40gmail.com", "only email was passed"),
        ("name=creatrixity", "only name was passed"),
        ("", "no data was passed"),
    ];

    for (value, reason) in test_cases {
        let response = app.post_subscriptions(value.into()).await;

        assert_eq!(
            400,
            response.status().as_u16(),
            "Failed with a 400 Bad Request because {}",
            reason
        );
    }
}

#[actix_rt::test]
async fn subscriber_creation_passes_when_fields_are_present_but_empty() {
    let app = utils::spawn_app().await;

    let test_cases = vec![
        ("name=kay&email=", "empty email"),
        ("name=kay&email=yo-not-an-email", "invalid email"),
    ];

    for (value, reason) in test_cases {
        let response = app.post_subscriptions(value.into()).await;

        assert_eq!(
            400,
            response.status().as_u16(),
            "The API Request did not return a 200 OK when payload was {}",
            reason
        );
    
    }
}

#[actix_rt::test]
async fn subscribe_sends_a_confirmation_email_for_valid_subscriptions () {
    let app = utils::spawn_app().await;
    let body = "email=creatrixity%40gmail.com&name=caleb";

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&app.email_server)
        .await;
    
    let response = app.post_subscriptions(body.into()).await;

    assert_eq!(200, response.status().as_u16());
}

#[actix_rt::test]
async fn subscribe_sends_a_confirmation_email_with_a_link() {
    let app = utils::spawn_app().await;
    let body = "email=creatrixity%40gmail.com&name=caleb";

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&app.email_server)
        .await;
    
    app.post_subscriptions(body.to_string()).await;

    let email_request = &app.email_server.received_requests().await.unwrap()[0];
    let confirmation_link = &app.get_confirmation_links(email_request.to_owned());

    assert_eq!(confirmation_link.html, confirmation_link.plain_text);
}