#![feature(once_cell)]
#![feature(iter_intersperse)]

use std::lazy::SyncLazy;
use std::sync::Arc;
use reqwest::{Client, Url};
use reqwest::cookie::Jar;
use scraper::{Html, Selector};
use serde::{Serialize, Deserialize};

pub mod submit;
pub mod problem;
pub mod status;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CsrfKey(pub String);

#[derive(Clone, Debug, Default, PartialEq)]
pub struct LoginCookie {
    pub boj_auto_login: String,
    pub online_judge: String
}

impl LoginCookie {
    pub fn apply(&self) {
        let url: Url = "https://www.acmicpc.net/".parse().unwrap();
        COOKIE.add_cookie_str(&format!("bojautologin={}", self.boj_auto_login), &url);
        COOKIE.add_cookie_str(&format!("OnlineJudge={}", self.online_judge), &url)
    }
}

pub static COOKIE: SyncLazy<Arc<Jar>> = SyncLazy::new(|| Arc::new(Jar::default()));

pub static CLIENT: SyncLazy<Client> = SyncLazy::new(|| {
    Client::builder()
        .cookie_store(true)
        .cookie_provider(COOKIE.clone())
        .build()
        .unwrap()
});

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[repr(u32)]
pub enum ProgrammingLanguageIds {
    Cpp17 = 84,
    Python3 = 28,
    PyPy3 = 73,
    C99 = 0,
    Java11 = 93,
    Ruby = 68,
    KotlinJvm = 69,
    Swift = 74,
    Text = 58,
    Cs90DotNet = 86,
    NodeJs = 17,
    Go = 12,
    D = 29,
    Rust2018 = 94,
    Cpp17Clang = 85
}

#[derive(Clone, Debug)]
pub struct Main(Html);

impl Main {
    pub async fn request() -> Result<Self, reqwest::Error> {
        Ok(Self(Html::parse_document(
            CLIENT.get("https://www.acmicpc.net/").send().await?.text().await?.as_str()
        )))
    }
    pub fn get_username(&self) -> &str {
        let selector = Selector::parse(r#"a[class="username"]"#).unwrap();
        self.0.select(&selector).next().unwrap().text().next().unwrap()
    }

}

#[cfg(test)]
mod tests {
    use crate::{LoginCookie, Main};
    use crate::problem::ProblemId;
    use crate::status::{Status, StatusRequest};
    use crate::status;

    #[tokio::test]
    async fn test1() {
        let login = LoginCookie {
            boj_auto_login: "aaaaa".to_string(),
            online_judge: "saafa".to_string()
        };
        login.apply();
        let main = Main::request().await.unwrap();
        println!("{}", main.get_username());
        let problem_id = ProblemId(10171);
        let status_request = StatusRequest {
            problem_id: Some(problem_id),
            user_id: Some(main.get_username().to_string())
        };

        let status = Status::request(&status_request).await.unwrap();
        let solution_id = status.get_solutions()[0].solution_id;
        println!("{:#?}", status::ajax(solution_id).await);
    }
}


