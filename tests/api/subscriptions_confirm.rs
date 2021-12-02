use reqwest::Url;
use linkify::LinkFinder;
use crate::utils;
use wiremock::{ ResponseTemplate, Mock };
use wiremock::matchers::{path, method};

#[actix_rt::test]
pub async fn the_link_returned_by_subscribe_returns_a_200_when_called() {
    // Arrange
    let app = utils::spawn_app().await;
    let body = "email=creatrixity%40gmail.com&name=caleb";

    // Act
    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&app.email_server)
        .await;
    
    app.post_subscriptions(body.into()).await;
    let email_request = &app.email_server.received_requests().await.unwrap()[0];
    let body: serde_json::Value = serde_json::from_slice(&email_request.body).unwrap();

    let get_link = |s: &str| {
        let links: Vec<_> = linkify::LinkFinder::new()
            .links(s)
            .filter(|l| *l.kind() == linkify::LinkKind::Url)
            .collect();

        assert_eq!(links.len(), 1);
        links[0].as_str().to_owned()
    };

    let raw_confirmation_link = &get_link(&body["HtmlBody"].as_str().unwrap());
    let mut confirmation_link = Url::parse(raw_confirmation_link).unwrap();

    // Assert
    // Ensure we only call the local machine.
    assert_eq!(&confirmation_link.host_str().unwrap().to_string(), "127.0.0.1");
    confirmation_link.set_port(Some(app.port)).unwrap();

    let response = reqwest::get(confirmation_link)
        .await
        .unwrap();
    
    assert_eq!(response.status().as_u16(), 200);
}