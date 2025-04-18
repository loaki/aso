use crate::{ app::{ App, Mode }, utils::timestamp_to_elapsed };
use tui::{
    backend::Backend,
    Terminal,
    Frame,
    widgets::{ List, ListItem, Paragraph, Wrap },
    layout::{ Layout, Constraint, Direction },
    text::{ Span, Spans, Text },
    style::{ Style, Color, Modifier },
};
use std::time::{ Duration, Instant };
use html2text::from_read;

pub fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
    tick_rate: Duration
) -> std::io::Result<()> {
    let mut last_tick = Instant::now();
    loop {
        terminal.draw(|f| draw_ui(f, &mut app))?;

        let timeout = tick_rate.checked_sub(last_tick.elapsed()).unwrap_or_default();
        if crossterm::event::poll(timeout)? {
            if let crossterm::event::Event::Key(key) = crossterm::event::read()? {
                use crossterm::event::KeyCode::*;
                match key.code {
                    Char('q') | Esc => {
                        return Ok(());
                    }
                    Up => app.on_up(),
                    Down => app.on_down(),
                    PageUp => app.on_page_up(),
                    PageDown => app.on_page_down(),
                    Left | Backspace => app.on_back(),
                    Right | Enter => app.on_enter(),
                    _ => {}
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            last_tick = Instant::now();
        }
    }
}

pub fn draw_ui<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let full_area = f.size();

    match &app.mode {
        Mode::Questions => {
            let items: Vec<ListItem> = app.items
                .iter()
                .enumerate()
                .map(|(index, i)| {
                    let number_width = (index + 1).to_string().len() + 4;
                    let available_width = (full_area.width as usize) - number_width;
                    let title = from_read(i.title.as_bytes(), available_width);
                    let truncated_title = if title.len() > available_width {
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
