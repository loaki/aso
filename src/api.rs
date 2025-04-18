use crate::models::{ QuestionsResponse, Question, AnswersResponse, Answer };
use std::error::Error;
use reqwest::header;

pub fn fetch_stackoverflow_questions(query: &str) -> Result<Vec<Question>, Box<dyn Error>> {
    let client = reqwest::blocking::Client::new();
    let url =
        format!("https://api.stackexchange.com/2.3/search/advanced?pagesize=20&order=desc&sort=activity&answers=1&site=stackoverflow&q={}&filter=!nNPvSNPI7A", query);

    let mut headers = header::HeaderMap::new();
    headers.insert("User-Agent", header::HeaderValue::from_static("tui-rs-app/0.1"));

    let resp = client.get(&url).headers(headers).send()?;
    if !resp.status().is_success() {
        return Err(format!("Request failed with status: {}", resp.status()).into());
    }

    let questions_response: QuestionsResponse = resp.json()?;
    Ok(questions_response.items)
}

pub fn fetch_stackoverflow_answers(question_id: u32) -> Result<Vec<Answer>, Box<dyn Error>> {
    let client = reqwest::blocking::Client::new();
    let url =
        format!("https://api.stackexchange.com/2.3/questions/{}/answers?order=desc&sort=activity&site=stackoverflow&filter=!nNPvSNdWme", question_id);

    let mut headers = header::HeaderMap::new();
    headers.insert("User-Agent", header::HeaderValue::from_static("tui-rs-app/0.1"));

    let resp = client.get(&url).headers(headers).send()?;
    if !resp.status().is_success() {
        return Err(format!("Request failed with status: {}", resp.status()).into());
    }

    let answers_response: AnswersResponse = resp.json()?;
    Ok(answers_response.items)
}
