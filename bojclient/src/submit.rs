use scraper::{Html, Selector};
use crate::problem::ProblemId;
use serde::{Serialize, Deserialize};
use crate::{CLIENT, CsrfKey};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SubmitRequestForm {
	pub recaptcha_response: String,
	pub problem_id: ProblemId,
	pub language: u32, //TODO make languages enum
	pub code_open: CodeVisibility,
	pub source: String,
	pub csrf_key: CsrfKey
}

impl SubmitRequestForm {
	pub async fn submit(&self, problem_id: ProblemId) -> Result<(), reqwest::Error> {
		CLIENT.post(get_url(problem_id)).form(&self).send().await.map(|_| ())
	}

}

#[derive(Clone, Debug)]
pub struct Submit(Html);

impl Submit {
	pub async fn request(problem_id: ProblemId) -> Result<Self, reqwest::Error> {
		Ok(Self(
			Html::parse_document(CLIENT.get(get_url(problem_id)).send().await?.text().await?.as_str())
		))
	}

	pub async fn get_csrf_key(&self) -> Result<CsrfKey, reqwest::Error> {
		let selector = Selector::parse(r#"input[name="csrf_key"]"#).unwrap();
		Ok(CsrfKey(
			self.0.select(&selector).next().unwrap().value().attr("value").unwrap()
				.parse()
				.unwrap()
		))
	}
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum CodeVisibility {
	#[serde(rename = "open")]
	Open,
	#[serde(rename = "close")]
	Close,
	#[serde(rename = "onlyaccepted")]
	OnlyAccepted
}

fn get_url(problem_id: ProblemId) -> String {
	format!("https://www.acmicpc.net/submit/{}", problem_id.0)
}

