#![feature(let_chains)]

use std::error::Error;

#[derive(Debug, Clone)]
pub struct TellmeClient{
    url     : url::Url,
    login   : Option<String>,
    password: Option<String>,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct Identifier{
    identifier: String,
}

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug)]
pub struct Service {
    pub service_type         : String,
    pub available            : bool,
    pub healthcheck_endpoint : String,
    pub is_accepted          : bool,
    pub identifier           : String,
    pub ip                   : url::Url,
}

impl TellmeClient {
    pub fn new(url: url::Url, login: Option<String>, password: Option<String>) -> Self {
        Self { url, login, password }
    }

    pub async fn register(&self,
                          port: u16,
                          healthcheck_endpoint: String,
                          access_token: String,
                          service_type: String
                        ) -> Result<String, Box<dyn Error>> {
        let registration_endpoint = self.url.join("/me")?;
        let params   = [("healthcheck_endpoint", healthcheck_endpoint),
                        ("access_token",         access_token),
                        ("service_type",         service_type),
                        ("port",                 port.to_string())];
        let client   = reqwest::Client::new();
        let response = client.post(registration_endpoint.to_string())
                             .form(&params)
                             .send()
                             .await?
                             .error_for_status()?;
        let answer   = response.json::<Identifier>().await?;

        Ok(answer.identifier)
    }

    pub async fn accept_service(&self, identifier: String) -> Result<(), Box<dyn Error>>{
        if let Some(login) = &self.login &&
           let Some(password) = &self.password{

            let accept_endpoint = self.url.join("/accept_service")?;
            let params = [("identifier", identifier),
                          ("login",      login.clone()),
                          ("password",   password.clone())];
            let client = reqwest::Client::new();
            let _ = client.post(accept_endpoint.to_string())
                          .form(&params)
                          .send()
                          .await?
                          .error_for_status()?;
            return Ok(());
        }
        Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Login and password must be the set!")))
    }

    pub async fn disable_service(&self, identifier: String) -> Result<(), Box<dyn Error>>{
        if let Some(login)    = &self.login    &&
           let Some(password) = &self.password {
            let disable_endpoint = self.url.join("/disable_service")?;
            let params = [("identifier", identifier),
                          ("login",      login.clone()),
                          ("password",   password.clone())];
            let client = reqwest::Client::new();
            let _ = client.post(disable_endpoint.to_string())
                          .form(&params)
                          .send()
                          .await?
                          .error_for_status()?;
            return Ok(());
        }
        Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Login and password must be the set!")))
    }

    pub async fn newtoken(&self) -> Result<String, Box<dyn Error>>{
        #[derive(serde::Deserialize)]
        struct Token {token: String}

        if let Some(login)    = &self.login    &&
           let Some(password) = &self.password {
            let newtoken_endpoint = self.url.join("/newtoken")?;
            let params   = [("login",    login.clone()),
                            ("password", password.clone())];
            let client   = reqwest::Client::new();
            let token = client.post(newtoken_endpoint.to_string())
                              .form(&params)
                              .send()
                              .await?
                              .error_for_status()?.json::<Token>().await?;
            return Ok(token.token);
     }
     Err(Box::new(std::io::Error::new(
                 std::io::ErrorKind::Other,
                 "Login and password must be the set!")))
    }

    pub async fn find(
            &self,
            service_type: Option<String>,
            limit       : Option<usize>,
            available   : Option<bool>
        ) -> Result<Vec<Service>, Box<dyn Error>> {

        let mut query_params = vec![];

        if let Some(service_type) = service_type{
            query_params.push(("service_type", service_type));
        }
        if let Some(limit) = limit{
            query_params.push(("limit", limit.to_string()));
        }
        if let Some(available) = available{
            query_params.push(("available", available.to_string()));
        }

        let find_endpoint = self.url.join("/find")?;
        let client = reqwest::Client::new();
        let response = client.get(find_endpoint.to_string()).query(&query_params).send().await?;
        let services = response.json::<Vec<Service>>().await?;

        Ok(services)
    }

    pub async fn subscribe(
        &self,
        identifier     : String,
        on_registration: bool,
        on_acceptance  : bool,
        endpoint       : String
        ) -> Result<(), Box<dyn Error>>{

        if let Some(login)    = &self.login   &&
           let Some(password) = &self.password {

            let newtoken_endpoint = self.url.join("/subscribe")?;
            let params   = [("login",           login.clone()),
                            ("password",        password.clone()),
                            ("identifier",      identifier),
                            ("endpoint",        endpoint),
                            ("on_registration", on_registration.to_string()),
                            ("on_acceptance",   on_acceptance.to_string())];
            let client   = reqwest::Client::new();
            let _ = client.post(newtoken_endpoint.to_string())
                          .form(&params)
                          .send()
                          .await?
                          .error_for_status()?;
            return Ok(());
     }
     Err(Box::new(std::io::Error::new(
                 std::io::ErrorKind::Other,
                 "Login and password must be the set!")))
    }
}

#[cfg(test)]
mod tests {
    use crate::*;
    #[test]
    fn it_works() {
        let my_port = 4567;

        let client = TellmeClient::new(
            url::Url::parse("http://localhost:8080").expect("Failed to parse service registry url"),
            Some(String::from("login")),
            Some(String::from("password"))
        );

        let access_token = client.newtoken()
                                 .await
                                 .expect("Failed to get access token!");

        let identifier = client.register(my_port,
                                         "/healthcheck_endpoint".to_owned(),
                                         access_token,
                                         "storage node".to_owned())
                                         .await.expect("Failed to register self in service registry!");

        // As we have login and password
        // we can accept self in service registry
        // Otherwise we need to wait someone to accept us

        client.accept_service(identifier)
              .await
              .expect("Failed to accept self in service registry");

        // We can also disable service if we need
        client.disable_service(identifier)
              .await
              .expect("Failed to disable self in service registry");

        // We can register endpoint for service registration and service acceptance
        client.subscribe(identifier, true, false, "/hook/on_registration")
              .await
              .expect("Failed to subscribe to service registration");

        client.subscribe(identifier, false, true, "/hook/on_acceptance")
              .await
              .expect("Failed to subscribe to service acceptance");

        /*
           Example of actix-web endpoint

           #[actix-web::post("/hook/on_registration")]
           async fn on_registration(service_data: actix_web::web::Form<tellme_client::Service) -> impl actix_web::Responder {
              actix_web::HttpResponse::Accepted().finish()
           }
        */



    }
}
