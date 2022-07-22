use http::status::StatusCode;
use http::Uri;
use isahc::{auth::Authentication, prelude::*, HttpClient, Request};
use rpassword::prompt_password;
use serde::{de, Deserialize, Serialize};
use serde_json;
use std::fmt;
use std::io::Write;
use std::ops::DerefMut;
use std::process::{Command, Stdio};
use std::sync::Arc;
use std::sync::Mutex;
use url::Url;
use users::get_current_username;

pub struct API {
  token: Arc<Mutex<Option<String>>>,
  api_base_url: String,
}

#[derive(Debug)]
pub enum APIError {
  Unauthorized,
  BadFormat,
  HTTPError(http::Error),
  IsahcError(isahc::Error),
  ServerError(Option<Uri>, String),
}

#[derive(Deserialize, Debug, Clone)]
struct ErrorResponse {
  error: String,
}

#[derive(Deserialize, Debug, Clone)]
struct MessageResponse {
  message: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct DrinkList {
  pub machines: Vec<Machine>,
  pub message: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Machine {
  pub display_name: String,
  pub id: u64,
  pub is_online: bool,
  pub name: String,
  pub slots: Vec<Slot>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Slot {
  pub active: bool,
  pub count: Option<u64>,
  pub empty: bool,
  pub item: Item,
  pub machine: u64,
  pub number: u8,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Item {
  pub id: u64,
  pub name: String,
  pub price: u64,
}

#[derive(Deserialize, Debug, Clone)]
struct User {
  preferred_username: String,
}

#[derive(Deserialize, Debug, Clone)]
struct CreditResponse {
  user: CreditUser,
}

#[allow(non_snake_case)]
#[derive(Deserialize, Debug, Clone)]
struct CreditUser {
  #[serde(deserialize_with = "number_string_deserializer")]
  drinkBalance: i64,
}

fn number_string_deserializer<'de, D>(deserializer: D) -> Result<i64, D::Error>
where
  D: de::Deserializer<'de>,
{
  let number: String = Deserialize::deserialize(deserializer)?;
  match number.parse::<i64>() {
    Ok(res) => Ok(res),
    Err(e) => Err(de::Error::custom(format!(
      "Failed to deserialize i64: {}",
      e
    ))),
  }
}

#[derive(Serialize, Debug, Clone)]
struct DropRequest {
  machine: String,
  slot: u8,
}

#[allow(non_snake_case)]
#[derive(Deserialize, Debug, Clone)]
struct DropResponse {
  drinkBalance: i64,
  // message: String,
}

impl std::error::Error for APIError {}

impl fmt::Display for APIError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      APIError::Unauthorized => write!(
        f,
        "Unauthorized (Did your Kerberos ticket expire?: `kinit`)"
      ),
      APIError::BadFormat => write!(f, "BadFormat (The server sent data we didn't understand)"),
      APIError::ServerError(path, message) => write!(
        f,
        "ServerError for {}: {}",
        match path {
          Some(ref uri) => uri.to_string(),
          None => "<unknown>".to_string(),
        },
        message
      ),
      APIError::HTTPError(err) => write!(f, "HTTPError: {}", err),
      APIError::IsahcError(err) => write!(f, "IsahcError: {}", err),
    }
  }
}

impl Default for API {
  fn default() -> Self {
    Self::new("https://drink.csh.rit.edu".to_string())
  }
}

enum APIBody<T: Serialize> {
  Json(T),
  NoBody,
}

impl<T: Serialize> From<APIBody<T>> for isahc::Body {
  fn from(body: APIBody<T>) -> Self {
    match body {
      APIBody::Json(value) => serde_json::to_string(&value).unwrap().into(),
      APIBody::NoBody => ().into(),
    }
  }
}

impl Clone for API {
  fn clone(&self) -> Self {
    Self {
      token: Arc::clone(&self.token),
      api_base_url: self.api_base_url.clone(),
    }
  }
}

impl API {
  pub fn new(api_base_url: String) -> API {
    // We should find a way to spin this off in a thread
    // api.get_token().ok();
    API {
      token: Arc::new(Mutex::new(None)),
      api_base_url,
    }
  }
  fn authenticated_request<O, I>(
    &self,
    builder: http::request::Builder,
    input: APIBody<I>,
  ) -> Result<O, APIError>
  where
    I: Serialize,
    O: de::DeserializeOwned,
  {
    let client = HttpClient::new().map_err(APIError::IsahcError)?;
    let token = self.get_token()?;
    let builder = builder
      .header("Authorization", token)
      .header("Accept", "application/json");
    let builder = match input {
      APIBody::Json(_) => builder.header("Content-Type", "application/json"),
      APIBody::NoBody => builder,
    };
    let mut response = client
      .send(builder.body(input).map_err(APIError::HTTPError)?)
      .map_err(APIError::IsahcError)?;
    match response.status() {
      StatusCode::OK => match response.json::<O>() {
        Ok(value) => Ok(value),
        Err(_) => Err(APIError::BadFormat),
      },
      _ => {
        let text = response.text().map_err(|_| APIError::BadFormat)?;
        let text_ref = &text;
        Err(APIError::ServerError(
          response.effective_uri().cloned(),
          serde_json::from_str::<ErrorResponse>(&text)
            .map(|body| body.error)
            .or_else(move |_| {
              serde_json::from_str::<MessageResponse>(text_ref).map(|body| body.message)
            })
            .unwrap_or(text),
        ))
      }
    }
  }
  pub fn drop(&self, machine: String, slot: u8) -> Result<i64, APIError> {
    self
      .authenticated_request::<DropResponse, _>(
        Request::post(format!("{}/drinks/drop", self.api_base_url)),
        APIBody::Json(DropRequest { machine, slot }),
      )
      .map(|drop| drop.drinkBalance)
  }

  fn take_token(&self, token: &mut Option<String>) -> Result<String, APIError> {
    match token {
      Some(token) => Ok(token.to_string()),
      None => {
        let response = Request::get("https://sso.csh.rit.edu/auth/realms/csh/protocol/openid-connect/auth?client_id=clidrink&redirect_uri=drink%3A%2F%2Fcallback&response_type=token%20id_token&scope=openid%20profile%20drink_balance&state=&nonce=")
          .authentication(Authentication::negotiate())
          .body(()).map_err(APIError::HTTPError)?.send().map_err(APIError::IsahcError)?;
        let location = match response.headers().get("Location") {
          Some(location) => location,
          None => {
            API::login();
            return self.take_token(token);
          }
        };
        let url = Url::parse(
          &location
            .to_str()
            .map_err(|_| APIError::BadFormat)?
            .replace('#', "?"),
        )
        .map_err(|_| APIError::BadFormat)?;

        for (key, value) in url.query_pairs() {
          if key == "access_token" {
            let value = format!("Bearer {}", value);
            *token = Some(value.clone());
            return Ok(value);
          }
        }
        Err(APIError::BadFormat)
      }
    }
  }

  pub fn get_token(&self) -> Result<String, APIError> {
    let mut token = self.token.lock().unwrap();
    self.take_token(token.deref_mut())
  }

  fn login() {
    // Get credentials
    let username: String = std::env::var("CLINK_USERNAME")
      .ok()
      .or_else(|| get_current_username().and_then(|username| username.into_string().ok()))
      .or_else(|| std::env::var("USER").ok())
      .expect("Couldn't determine username");

    // Start kinit, ready to get password from pipe
    let mut process = Command::new("kinit")
      .arg(format!("{}@CSH.RIT.EDU", username))
      .stdin(Stdio::piped())
      .stdout(Stdio::null())
      .spawn()
      .unwrap();

    // Get password
    let password = prompt_password(format!("Password for {}: ", username)).unwrap();

    // Pipe password into the prompt that "comes up"
    process
      .stdin
      .as_ref()
      .unwrap()
      .write_all(password.as_bytes())
      .unwrap();

    // Wait for login to be complete before continuting
    process.wait().unwrap();
  }

  pub fn get_credits(&self) -> Result<i64, APIError> {
    // Can also be used to get other user information
    let user: User = self.authenticated_request(
      Request::get("https://sso.csh.rit.edu/auth/realms/csh/protocol/openid-connect/userinfo"),
      APIBody::NoBody as APIBody<serde_json::Value>,
    )?;
    let credit_response: CreditResponse = self.authenticated_request(
      Request::get(format!(
        "{}/users/credits?uid={}",
        self.api_base_url, user.preferred_username
      )),
      APIBody::NoBody as APIBody<serde_json::Value>,
    )?;
    Ok(credit_response.user.drinkBalance)
  }

  pub fn get_status_for_machine(&self, machine: Option<&str>) -> Result<DrinkList, APIError> {
    self.authenticated_request(
      Request::get(format!(
        "{}/drinks{}",
        self.api_base_url,
        match machine {
          Some(machine) => format!("?machine={}", machine),
          None => "".to_string(),
        }
      )),
      APIBody::NoBody as APIBody<serde_json::Value>,
    )
  }
}
