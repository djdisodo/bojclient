use std::fs::read_to_string;
use std::path::Path;
use std::time::Duration;
use clap::{App, Arg};
use yaml_rust::YamlLoader;
use bojclient::{LoginCookie, Main, ProgrammingLanguageIds};
use bojclient::problem::ProblemId;
use bojclient::status::{SolutionResult, Status, StatusRequest};
use bojclient::submit::{CodeVisibility, Submit};

#[tokio::main]
async fn main() {
    let app = App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"));
    let matches = app
        .arg(
            Arg::with_name("PROBLEM_ID")
                .help("id of problem, digits")
                .required(true)
                .index(1)
        )
        .arg(
            Arg::with_name("wait")
                .short("w")
                .long("wait")
                .help("wait and return the result")
        ).get_matches();
    let problem_id = ProblemId(matches.value_of("PROBLEM_ID").unwrap().parse().expect("problem id should be digits"));
    let config = read_config(std::env::current_dir().unwrap().as_path());
    config.login_cookie.apply();
    let language: ProgrammingLanguageIds = serde_plain::from_str(&config.language).unwrap();
    println!("reading source code file...");
    let source_code = read_to_string(config.target_file).unwrap();

    println!("fetching user id...");

    let user_id = Main::request().await.unwrap().get_username().to_owned();

    println!("user id: {}", user_id);

    println!("requesting csrf_key...");
    let csrf_key = Submit::request(problem_id).await.unwrap().get_csrf_key().await.unwrap();
    println!("csrf_key: {}", csrf_key.0);

    let submit_form = bojclient::submit::SubmitRequestForm {
        recaptcha_response: "".to_string(),
        problem_id,
        language: language as u32,
        code_open: CodeVisibility::Open,
        source: source_code,
        csrf_key
    };

    println!("sending submit request...");

    submit_form.submit(problem_id).await.unwrap();

    println!("finding solution id...");
    let solutions = Status::request(&StatusRequest {
        problem_id: Some(problem_id),
        user_id: Some(user_id)
    }).await.unwrap().get_solutions();

    let current_solution = &solutions[0];

    println!("solution id: {}", current_solution.solution_id);

    if matches.occurrences_of("wait") == 0 {
        return;
    }

    let mut result = current_solution.result;

    while result == SolutionResult::Other {
        println!("evaluating...");
        tokio::time::sleep(Duration::from_secs(1)).await;
        result = bojclient::status::ajax(current_solution.solution_id).await.unwrap().result_name
    }

    println!("result: {:?}", result)

}

#[derive(Clone, Debug, Default)]
struct SubmitConfig {
    login_cookie: LoginCookie,
    pre_run: String,
    run: String,
    target_file: String,
    language: String
}

fn read_config(path: &Path) -> SubmitConfig {
    let mut config = if let Some(parent) = path.parent() {
        read_config(parent)
    } else {
        SubmitConfig::default()
    };
    let mut path_buf = path.to_path_buf();
    path_buf.push(".bc.yml");
    let yml_path = path_buf.as_path();
    if let Ok(data) = read_to_string(yml_path) {
        match YamlLoader::load_from_str(&data) {
            Ok(docs) => {
                let doc = &docs[0];
                config.login_cookie.boj_auto_login = doc["boj_auto_login"].as_str().unwrap_or(&config.login_cookie.boj_auto_login).to_string();
                config.login_cookie.online_judge = doc["online_judge"].as_str().unwrap_or(&config.login_cookie.online_judge).to_string();
                config.pre_run = doc["pre_run"].as_str().unwrap_or(&config.pre_run).to_string();
                config.run = doc["run"].as_str().unwrap_or(&config.run).to_string();
                config.target_file = doc["target_file"].as_str().unwrap_or(&config.target_file).to_string();
                config.language = doc["language"].as_str().unwrap_or(&config.language).to_string();
            }
            Err(e) => eprintln!("{:#?}", e)
        }
    }
    config
}