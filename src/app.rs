use crate::api::fetch_stackoverflow_answers;
use crate::models::{Answer, Owner, Question};
use tui::widgets::ListState;

pub enum Mode {
    Questions,
    Answers(AnswersView),
}

pub struct AnswersView {
    pub question: Question,
    pub answers: Vec<Answer>,
    pub scroll: u16,
}

pub struct App {
    pub items: Vec<Question>,
    pub query: String,
    pub state: ListState,
    pub mode: Mode,
}

impl App {
    pub fn new(items: Vec<Question>, query: String) -> Self {
        let mut state = ListState::default();
        state.select(Some(0));
        Self {
            items,
            query,
            state,
            mode: Mode::Questions,
        }
    }

    pub fn on_up(&mut self) {
        match &mut self.mode {
            Mode::Questions => {
                let i = match self.state.selected() {
                    Some(i) => if i == 0 { self.items.len() - 1 } else { i - 1 },
                    None => 0,
                };
                self.state.select(Some(i));
            }
            Mode::Answers(a) => if a.scroll > 0 { a.scroll -= 1 },
        }
    }

    pub fn on_down(&mut self) {
        match &mut self.mode {
            Mode::Questions => {
                let i = match self.state.selected() {
                    Some(i) => if i == self.items.len() - 1 { 0 } else { i + 1 },
                    None => 0,
                };
                self.state.select(Some(i));
            }
            Mode::Answers(a) => a.scroll += 1,
        }
    }

    pub fn on_page_up(&mut self) {
        match &mut self.mode {
            Mode::Questions => {
                let i = match self.state.selected() {
                    Some(i) => if i <= 5 { 0 } else { i - 5 },
                    None => 0,
                };
                self.state.select(Some(i));
            }
            Mode::Answers(a) => a.scroll = a.scroll.saturating_sub(5),
        }
    }

    pub fn on_page_down(&mut self) {
        match &mut self.mode {
            Mode::Questions => {
                let i = match self.state.selected() {
                    Some(i) => if i + 5 >= self.items.len() { self.items.len() - 1 } else { i + 5 },
                    None => 0,
                };
                self.state.select(Some(i));
            }
            Mode::Answers(a) => a.scroll += 5,
        }
    }

    pub fn on_enter(&mut self) {
        if let Mode::Questions = self.mode {
            if let Some(i) = self.state.selected() {
                let question = &self.items[i];
                self.mode = Mode::Answers(AnswersView {
                    question: question.clone(),
                    answers: fetch_stackoverflow_answers(question.question_id).unwrap_or_else(|_| vec![
                        Answer {
                            owner: Owner {
                                display_name: "Error".to_string(),
                            },
                            body: "Failed to fetch answers.".to_string(),
                            creation_date: 0,
                        }
                    ]),
                    scroll: 0,
                });
            }
        }
    }

    pub fn on_back(&mut self) {
        if let Mode::Answers(_) = self.mode {
            self.mode = Mode::Questions;
        }
    }
}
