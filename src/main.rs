use crossterm::{
    event::{ self, KeyCode },
    execute,
    terminal::{ disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen },
};
use std::{ error::Error, io, time::{ Duration, Instant } };
use tui::{
    backend::{ Backend, CrosstermBackend },
    style::{ Color, Modifier, Style },
    text::{ Span, Spans },
    widgets::{ List, ListItem, ListState, Paragraph },
    Frame,
    Terminal,
};
use reqwest::header;
use chrono::{ TimeZone, Utc };
use html2text::from_read;
use tui::widgets::Wrap;
use tui::text::{ Text };
use tui::layout::{ Constraint, Direction, Layout };

#[derive(serde::Deserialize)]
struct QuestionsResponse {
    items: Vec<Question>,
}

#[derive(serde::Deserialize, Clone)]
struct Question {
    title: String,
    body: String,
    question_id: u32,
    creation_date: u64,
    answer_count: u32,
    owner: Owner,
    link: String,
}

#[derive(serde::Deserialize)]
struct AnswersResponse {
    items: Vec<Answer>,
}

#[derive(serde::Deserialize)]
struct Answer {
    body: String,
    owner: Owner,
    creation_date: u64,
}

#[derive(serde::Deserialize, Clone)]
struct Owner {
    display_name: String,
}

enum Mode {
    Questions,
    Answers(AnswersView),
}

struct AnswersView {
    question: Question,
    answers: Vec<Answer>,
    scroll: u16,
}

struct App {
    items: Vec<Question>,
    query: String,
    state: ListState,
    mode: Mode,
}

impl App {
    fn new(items: Vec<Question>, query: String) -> App {
        let mut state = ListState::default();
        state.select(Some(0));
        App {
            items,
            query,
            state,
            mode: Mode::Questions,
        }
    }

    fn on_up(&mut self) {
        match &mut self.mode {
            Mode::Questions => {
                let i = match self.state.selected() {
                    Some(i) => if i == 0 { self.items.len() - 1 } else { i - 1 }
                    None => 0,
                };
                self.state.select(Some(i));
            }
            Mode::Answers(answers) => {
                if answers.scroll > 0 {
                    answers.scroll -= 1;
                }
            }
        }
    }

    fn on_down(&mut self) {
        match &mut self.mode {
            Mode::Questions => {
                let i = match self.state.selected() {
                    Some(i) => if i == self.items.len() - 1 { 0 } else { i + 1 }
                    None => 0,
                };
                self.state.select(Some(i));
            }
            Mode::Answers(answers) => {
                answers.scroll += 1;
            }
        }
    }

    fn on_page_up(&mut self) {
        match &mut self.mode {
            Mode::Questions => {
                let i = match self.state.selected() {
                    Some(i) => if i <= 5 { 0 } else { i - 5 }
                    None => 0,
                };
                self.state.select(Some(i));
            }
            Mode::Answers(answers) => {
                if answers.scroll > 5 {
                    answers.scroll -= 5;
                } else {
                    answers.scroll = 0;
                }
            }
        }
    }

    fn on_page_down(&mut self) {
        match &mut self.mode {
            Mode::Questions => {
                let i = match self.state.selected() {
                    Some(i) => if i + 5 >= self.items.len() { self.items.len() - 1 } else { i + 5 }
                    None => 0,
                };
                self.state.select(Some(i));
            }
            Mode::Answers(answers) => {
                answers.scroll += 5;
            }
        }
    }

    fn on_enter(&mut self) {
        if let Mode::Questions = self.mode {
            if let Some(i) = self.state.selected() {
                let question = &self.items[i];
                self.mode = Mode::Answers(AnswersView {
                    question: question.clone(),
                    answers: fetch_stackoverflow_answers(question.question_id).unwrap_or_else(|_| {
                        vec![Answer {
                            owner: Owner {
                                display_name: "Error".to_string(),
                            },
                            body: "Failed to fetch answers.".to_string(),
                            creation_date: 0,
                        }]
                    }),
                    scroll: 0,
                });
            }
        }
    }

    fn on_back(&mut self) {
        if let Mode::Answers(_) = self.mode {
            self.mode = Mode::Questions;
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <search-term>", args[0]);
        std::process::exit(1);
    }

    let query = &args[1];
    let items = fetch_stackoverflow_questions(query)?;

    enable_raw_mode()?;
    execute!(io::stdout(), EnterAlternateScreen)?;
    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let tick_rate = Duration::from_millis(200);
    let app = App::new(items, query.clone());
    let res = run_app(&mut terminal, app, tick_rate);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;

    if let Err(err) = res {
        eprintln!("{:?}", err);
    }

    Ok(())
}

fn fetch_stackoverflow_questions(query: &str) -> Result<Vec<Question>, Box<dyn Error>> {
    let client = reqwest::blocking::Client::new();
    let url =
        format!("https://api.stackexchange.com/2.3/search/advanced?pagesize=20&order=desc&sort=activity&answers=1&site=stackoverflow&q={}&filter=!nNPvSNPI7A", query);

    let mut headers = header::HeaderMap::new();
    headers.insert("User-Agent", header::HeaderValue::from_static("tui-rs-app/0.1"));

    let resp = client.get(&url).headers(headers).send()?;

    if !resp.status().is_success() {
        return Err(
            format!(
                "Request failed with status: {}\n{}",
                resp.status(),
                resp.text().unwrap_or_default()
            ).into()
        );
    }

    let body = resp.text()?;
    let questions_response: QuestionsResponse = serde_json::from_str(&body)?;
    Ok(questions_response.items)
}

fn fetch_stackoverflow_answers(question_id: u32) -> Result<Vec<Answer>, Box<dyn Error>> {
    let client = reqwest::blocking::Client::new();
    let url =
        format!("https://api.stackexchange.com/2.3/questions/{}/answers?order=desc&sort=activity&site=stackoverflow&filter=!nNPvSNdWme", question_id);

    let mut headers = header::HeaderMap::new();
    headers.insert("User-Agent", header::HeaderValue::from_static("tui-rs-app/0.1"));

    let resp = client.get(&url).headers(headers).send()?;

    if !resp.status().is_success() {
        return Err(
            format!(
                "Request failed with status: {}\n{}",
                resp.status(),
                resp.text().unwrap_or_default()
            ).into()
        );
    }

    let body = resp.text()?;
    let answers_response: AnswersResponse = serde_json::from_str(&body)?;
    Ok(answers_response.items)
}

fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
    tick_rate: Duration
) -> io::Result<()> {
    let mut last_tick = Instant::now();
    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));
        if crossterm::event::poll(timeout)? {
            if let crossterm::event::Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => {
                        return Ok(());
                    }
                    KeyCode::Up => app.on_up(),
                    KeyCode::Down => app.on_down(),
                    KeyCode::PageUp => app.on_page_up(),
                    KeyCode::PageDown => app.on_page_down(),
                    KeyCode::Left => app.on_back(),
                    KeyCode::Right => app.on_enter(),
                    KeyCode::Enter => app.on_enter(),
                    KeyCode::Backspace => app.on_back(),
                    _ => {}
                }
            }
        }
        if last_tick.elapsed() >= tick_rate {
            last_tick = Instant::now();
        }
    }
}

fn timestamp_to_elapsed(timestamp: i64) -> String {
    let now = Utc::now();
    let then = Utc.timestamp_opt(timestamp, 0).unwrap();
    let duration = now.signed_duration_since(then);

    if duration.num_seconds() < 60 {
        "just now".to_string()
    } else if duration.num_minutes() < 60 {
        format!("{} minute{} ago", duration.num_minutes(), if duration.num_minutes() == 1 {
            ""
        } else {
            "s"
        })
    } else if duration.num_hours() < 24 {
        format!("{} hour{} ago", duration.num_hours(), if duration.num_hours() == 1 {
            ""
        } else {
            "s"
        })
    } else if duration.num_days() < 30 {
        format!("{} day{} ago", duration.num_days(), if duration.num_days() == 1 {
            ""
        } else {
            "s"
        })
    } else if duration.num_days() < 365 {
        let months = duration.num_days() / 30;
        format!("{} month{} ago", months, if months == 1 { "" } else { "s" })
    } else {
        let years = duration.num_days() / 365;
        format!("{} year{} ago", years, if years == 1 { "" } else { "s" })
    }
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let full_area = f.size();

    match &app.mode {
        Mode::Questions => {
            let items: Vec<ListItem> = app.items
                .iter()
                .enumerate()
                .map(|(index, i)| {
                    let number_width = (index + 1).to_string().len() + 2;
                    let available_width = (full_area.width as usize) - number_width;
                    let title = from_read(i.title.as_bytes(), available_width);
                    let truncated_title = if title.len() > available_width + 2 {
                        format!("{}...", &title[..available_width - 3])
                    } else {
                        title
                    };

                    let title_span = if Some(index) == app.state.selected() {
                        Spans::from(
                            vec![
                                Span::styled(
                                    format!("{}. ", index + 1),
                                    Style::default().fg(Color::LightYellow)
                                ),
                                Span::styled(
                                    truncated_title,
                                    Style::default().add_modifier(Modifier::REVERSED)
                                )
                            ]
                        )
                    } else {
                        Spans::from(
                            vec![
                                Span::styled(
                                    format!("{}. ", index + 1),
                                    Style::default().fg(Color::LightYellow)
                                ),
                                Span::styled(truncated_title, Style::default().fg(Color::White))
                            ]
                        )
                    };

                    let info_span = Spans::from(
                        vec![
                            Span::styled(
                                " ".repeat(number_width),
                                Style::default().fg(Color::White)
                            ),
                            Span::styled(
                                timestamp_to_elapsed(i.creation_date as i64),
                                Style::default().fg(Color::DarkGray)
                            ),
                            Span::styled(
                                format!(" · {} answers", i.answer_count),
                                Style::default().fg(Color::DarkGray)
                            )
                        ]
                    );
                    ListItem::new(vec![title_span, info_span])
                })
                .collect();

            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(2), Constraint::Min(0)])
                .split(full_area);

            let query_paragraph = Paragraph::new(
                Span::styled(
                    format!("Results for: {}", app.query),
                    Style::default().add_modifier(Modifier::BOLD)
                )
            ).style(Style::default().fg(Color::White));

            f.render_widget(query_paragraph, chunks[0]);

            let list = List::new(items)
                .style(Style::default().fg(Color::White))
                .highlight_symbol("> ");

            f.render_stateful_widget(list, chunks[1], &mut app.state);
        }

        Mode::Answers(answers) => {
            let available_width = (full_area.width as usize) - 4;

            let mut lines: Vec<Spans> = vec![
                Spans::from(
                    Span::styled(
                        from_read(answers.question.title.as_bytes(), available_width),
                        Style::default().add_modifier(Modifier::BOLD)
                    )
                ),
                Spans::from(
                    Span::styled(
                        answers.question.link.clone(),
                        Style::default().fg(Color::LightBlue)
                    )
                ),
                Spans::from(""),
                Spans::from(
                    vec![
                        Span::styled(
                            answers.question.owner.display_name.clone(),
                            Style::default().add_modifier(Modifier::BOLD)
                        ),
                        Span::raw(" · "),
                        Span::styled(
                            timestamp_to_elapsed(answers.question.creation_date as i64),
                            Style::default().fg(Color::DarkGray)
                        )
                    ]
                )
            ];

            let wrap_width = full_area.width.saturating_sub(4) as usize;
            lines.push(
                Spans::from(
                    Span::styled("─".repeat(wrap_width), Style::default().fg(Color::LightYellow))
                )
            );
            lines.extend(
                from_read(answers.question.body.as_bytes(), wrap_width)
                    .lines()
                    .map(|line| Spans::from(Span::raw(line.to_string())))
            );
            lines.push(Spans::from(""));

            for answer in &answers.answers {
                lines.push(
                    Spans::from(
                        vec![
                            Span::styled(
                                answer.owner.display_name.clone(),
                                Style::default().add_modifier(Modifier::BOLD)
                            ),
                            Span::raw(" · "),
                            Span::styled(
                                timestamp_to_elapsed(answer.creation_date as i64),
                                Style::default().fg(Color::DarkGray)
                            )
                        ]
                    )
                );
                lines.push(
                    Spans::from(
                        Span::styled("─".repeat(wrap_width), Style::default().fg(Color::LightGreen))
                    )
                );
                lines.extend(
                    from_read(answer.body.as_bytes(), wrap_width)
                        .lines()
                        .map(|line| Spans::from(Span::raw(line.to_string())))
                );

                lines.push(Spans::from(""));
            }

            let paragraph = Paragraph::new(Text::from(lines))
                .style(Style::default().fg(Color::White))
                .scroll((answers.scroll, 0))
                .wrap(Wrap { trim: false });

            f.render_widget(paragraph, full_area);
        }
    }
}
