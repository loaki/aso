#[derive(serde::Deserialize)]
pub struct QuestionsResponse {
    pub items: Vec<Question>,
}

#[derive(serde::Deserialize, Clone)]
pub struct Question {
    pub title: String,
    pub body: String,
    pub question_id: u32,
    pub creation_date: u64,
    pub answer_count: u32,
    pub owner: Owner,
    pub link: String,
}

#[derive(serde::Deserialize)]
pub struct AnswersResponse {
    pub items: Vec<Answer>,
}

#[derive(serde::Deserialize, Clone)]
pub struct Answer {
    pub body: String,
    pub owner: Owner,
    pub creation_date: u64,
}

#[derive(serde::Deserialize, Clone)]
pub struct Owner {
    pub display_name: String,
}
