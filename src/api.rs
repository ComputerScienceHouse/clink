extern crate rpassword;

use http::status::StatusCode;
use isahc::{auth::Authentication, prelude::*, HttpClient, Request};
use rpassword::read_password;
use serde_json::json;
use serde_json::{Map, Value};
use std::fmt;
use std::io::{Read, Write};
use std::process::{Command, Stdio};
use url::Url;

pub struct API {
  token: Option<String>,
}

#[derive(Debug)]
pub enum APIError {
  Unauthorized,
  BadFormat,
}

impl std::error::Error for APIError {}

impl fmt::Display for APIError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    return f.write_str(match self {
      APIError::Unauthorized => "Unauthorized (Did your Kerberos ticket expire?: `kinit`)",
      APIError::BadFormat => "BadFormat (The server sent data we didn't understand)",
    });
  }
}

impl API {
  pub fn new() -> API {
    return API { token: None };
  }
  pub fn drop(self: &mut API, machine: String, slot: u8) -> Result<(), Box<dyn std::error::Error>> {
    let token = self.get_token()?;

    let client = HttpClient::new()?;
    let request = Request::post("https://drink.csh.rit.edu/drinks/drop")
      .header("Content-Type", "application/json")
      .header("Authorization", token)
      .body(
        json!({
          "machine": machine,
          "slot": slot,
        })
        .to_string(),
      )?;
    let mut response = client.send(request)?;
    let body: Value = response.json()?;
    return match response.status() {
      StatusCode::OK => Ok(()),
      _ => {
        eprintln!("Couldn't drop: {}", body["error"].as_str().unwrap());
        return Err(Box::new(APIError::BadFormat));
      }
    };
  }

  pub fn get_token(self: &mut API) -> Result<String, Box<dyn std::error::Error>> {
    return match &self.token {
      Some(token) => Ok(token.to_string()),
      None => {
        let response = Request::get("https://sso.csh.rit.edu/auth/realms/csh/protocol/openid-connect/auth?client_id=clidrink&redirect_uri=drink%3A%2F%2Fcallback&response_type=token%20id_token&scope=openid%20profile%20drink_balance&state=&nonce=")
          .authentication(Authentication::negotiate())
          .body(())?.send()?;
        let location = match response.headers().get("Location") {
          Some(location) => location,
          None => {
            self.login();
            return self.get_token();
          },
        };
        let url = Url::parse(&location.to_str()?.replace("#", "?"))?;

        for (key, value) in url.query_pairs() {
          if key == "access_token" {
            let value = "Bearer ".to_owned() + &value.to_string();
            self.token = Some(value.to_string());
            return Ok(value.to_string());
          }
        }
        return Err(Box::new(APIError::BadFormat));
      }
    };
  }

  fn login(self: &mut API){
    // Get credentials
    let mut username = String::new();
    println!("Please enter your CSH username: ");
    std::io::stdin().read_line(&mut username).expect("Fugma");
    println!("Please enter your CSH password: ");
    let password = read_password().unwrap();
    // Start kinit, ready to get password from pipe
    let process = match Command::new("sh")
                                .arg("-c")
                                .arg(String::from("kinit ") + &String::from(username.trim()) + &String::from("@CSH.RIT.EDU"))
                                .stdin(Stdio::piped())
                                .stdout(Stdio::piped())
                                .spawn() {
        Err(why) => panic!("couldn't spawn kinit: {}", why),
        Ok(process) => process,
    };

    // Pipe in password
    match process.stdin.unwrap().write_all(password.as_bytes()) {
        Err(why) => panic!("couldn't write password: {}", why),
        Ok(_) => println!("..."),
    }

    let mut s = String::new();
    match process.stdout.unwrap().read_to_string(&mut s) {
        Err(why) => panic!("couldn't read stdout: {}", why),
        Ok(_) => (),
    }
  }

  pub fn get_credits(self: &mut API) -> Result<u64, Box<dyn std::error::Error>> {
    //let token = self.get_token()?;
    let client = HttpClient::new()?;
    // Can also be used to get other user information
    let request = Request::get("https://sso.csh.rit.edu/auth/realms/csh/protocol/openid-connect/userinfo")
        .header("Authorization", self.get_token()?)
        .body(())?;
    let response: Value = client.send(request)?.json()?;
    let uid = response["preferred_username"].as_str().unwrap().to_string();
    let credit_request = Request::get(format!("https://drink.csh.rit.edu/users/credits?uid={}", uid))
        .header("Authorization", self.get_token()?)
        .body(())?;
    let credit_response: Value = client.send(credit_request)?.json()?;
    Ok(credit_response["user"]["drinkBalance"].as_str().unwrap().parse::<u64>()?) // Coffee
  }

  pub fn get_machine_status(self: &mut API) -> Result<Value, Box<dyn std::error::Error>> {
    let token = self.get_token()?;
    let client = HttpClient::new()?;
    let request = Request::get("https://drink.csh.rit.edu/drinks")
      .header("Authorization", token)
      .body(())?;
    Ok(client.send(request)?.json()?)
  }
}
