use scraper::{Html, Selector};
use crate::problem::ProblemId;
use serde::{Serialize, Deserialize};
use crate::CLIENT;
use serde_aux::field_attributes::deserialize_number_from_string;
#[derive(Clone, Debug)]
pub struct Status(Html);

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StatusRequest {
	pub problem_id: Option<ProblemId>,
	pub user_id: Option<String>
}

impl Status {
	pub async fn request(status_request: &StatusRequest) -> Result<Self, reqwest::Error> {
		Ok(Self(Html::parse_document(
			CLIENT.get("https://www.acmicpc.net/status").query(&status_request).send().await?.text().await?.as_str()
		)))
	}

	pub fn get_solutions(&self) -> Vec<Solution> {
		let selector = Selector::parse(r#"tr[id^=solution-]"#).unwrap();
		self.0.select(&selector).map(|submission| {
			let td_selector = Selector::parse(r#"td"#).unwrap();
			let mut tds = submission.select(&td_selector);
			let solution_id: u32 = tds.next().unwrap().text().next().unwrap().parse().unwrap();
			let user: String = tds.next().unwrap().text().next().unwrap().to_owned();
			let problem_id: u32 = tds.next().unwrap().text().next().unwrap().parse().unwrap();
			let result: String = tds.next().unwrap().text().next().unwrap().to_owned();

			Solution {
				solution_id,
				user,
				problem_id: ProblemId(problem_id),
				result: serde_json::from_str(&format!("\"{}\"", result)).unwrap()
			}
		}).collect()
	}

}

#[derive(Clone, Debug)]
pub struct Solution {
	pub solution_id: u32,
	pub user: String,
	pub problem_id: ProblemId,
	pub result: SolutionResult,
	//pub memory: String,
	//pub time: String,

}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AjaxResponse {
	#[serde(deserialize_with = "deserialize_number_from_string")]
	pub solution_id: u32,
	#[serde(flatten)]
	pub resource_used: Option<ResourceUsed>,
	pub result_color: String,
	pub result_name: SolutionResult
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct ResourceUsed {
	#[serde(deserialize_with = "deserialize_number_from_string")]
	pub time: u32,
	#[serde(deserialize_with = "deserialize_number_from_string")]
	pub memory: u32
}

pub async fn ajax(solution_id: u32) -> Result<AjaxResponse, reqwest::Error> {
	#[derive(Serialize)]
	struct Form {
		solution_id: u32
	}
	Ok(CLIENT.post("https://www.acmicpc.net/status/ajax")
		.form(&Form {
			solution_id
		})
		.header("X-Requested-With", "XMLHttpRequest")
		.send().await?.json().await?)
}

#[derive(Copy, Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum SolutionResult {
	#[serde(alias = "맞았습니다!!")]
	Accepted,
	#[serde(alias = "출력 형식이 잘못되었습니다")]
	WrongFormat,
	#[serde(alias = "틀렸습니다")]
	Wrong,
	#[serde(alias = "시간 초과")]
	Timeout,
	#[serde(alias = "메모리 초과")]
	OutOfMemory,
	#[serde(alias = "출력 초과")]
	TooMuchOutput,
	#[serde(alias = "런타임 에러")]
	RuntimeError,
	#[serde(alias = "컴파일 에러")]
	CompileError,
	#[serde(other)]
	Other
}