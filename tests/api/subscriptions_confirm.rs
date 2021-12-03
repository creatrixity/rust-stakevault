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
    let confirmation_links = app.get_confirmation_links(email_request.to_owned());

    let response = reqwest::get(confirmation_links.html)
        .await
        .unwrap();
    
    assert_eq!(response.status().as_u16(), 200);
}