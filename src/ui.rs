use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use crate::{app::App, engine};

fn stack_hint_style() -> Style {
    Style::default().fg(Color::Rgb(170, 170, 170))
}

fn input_hint_style() -> Style {
    Style::default().fg(Color::Red)
}

fn input_approximation_hint_style() -> Style {
    Style::default().fg(Color::Rgb(170, 170, 170))
}

fn input_hint_spans(hint: &[crate::app::HintToken]) -> Vec<Span<'static>> {
    let mut spans = Vec::new();

    for (index, token) in hint.iter().enumerate() {
        if index > 0 {
            spans.push(Span::raw(" "));
        }

        spans.push(Span::styled(token.primary.clone(), input_hint_style()));
    }

    if hint.iter().any(|token| token.approximation.is_some()) {
        spans.push(Span::styled(" | ", input_approximation_hint_style()));

        for (index, token) in hint.iter().enumerate() {
            if index > 0 {
                spans.push(Span::raw(" "));
            }

            spans.push(Span::styled(
                token.approximation
                    .clone()
                    .unwrap_or_else(|| token.primary.clone()),
                input_approximation_hint_style(),
            ));
        }
    }

    spans
}

pub fn draw(frame: &mut Frame, app: &App) {
    let sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(3)])
        .split(frame.area());

    render_stack(frame, app, sections[0]);
    render_input(frame, app, sections[1]);
}

fn render_stack(frame: &mut Frame, app: &App, area: Rect) {
    let inner_height = area.height.saturating_sub(2) as usize;
    let visible_items = app.stack().len().min(inner_height);
    let title = format!("Stack ({visible_items} visible)");

    let mut lines = vec![Line::default(); inner_height.saturating_sub(visible_items)];
    let visible_slice = &app.stack()[app.stack().len().saturating_sub(visible_items)..];

    for (index, value) in visible_slice.iter().enumerate() {
        let line_number = visible_items - index;
        let formatted = engine::format_number_parts(value);
        let mut spans = vec![
            Span::styled(
                format!("{line_number}: "),
                Style::default().fg(Color::DarkGray),
            ),
            Span::raw(formatted.primary),
        ];

        if app.display_config().approximation_hint.stack {
            if let Some(approximation) = formatted.approximation {
                spans.push(Span::styled(
                    format!("  | {approximation}"),
                    stack_hint_style(),
                ));
            }
        }

        lines.push(Line::from(spans));
    }

    let stack_widget =
        Paragraph::new(lines).block(Block::default().borders(Borders::ALL).title(title));
    frame.render_widget(stack_widget, area);
}

fn render_input(frame: &mut Frame, app: &App, area: Rect) {
    let title = match app.status() {
        Some(status) => Line::from(vec![
            Span::raw("Input "),
            Span::styled(format!("[ {} ]", status), Style::default().fg(Color::Red)),
        ]),
        None => Line::from("Input"),
    };

    let block = Block::default().borders(Borders::ALL).title(title);
    let inner = block.inner(area);

    let mut spans = vec![Span::raw(app.input().to_string())];
    if let Some(hint) = app.hint() {
        spans.push(Span::raw("  "));
        spans.extend(input_hint_spans(&hint));
    }

    frame.render_widget(Paragraph::new(Line::from(spans)).block(block), area);

    if inner.width > 0 && inner.height > 0 {
        let max_offset = inner.width.saturating_sub(1) as usize;
        let cursor_offset = app.cursor().min(max_offset) as u16;
        frame.set_cursor_position((inner.x + cursor_offset, inner.y));
    }
}

#[cfg(test)]
mod tests {
    use super::input_hint_spans;
    use crate::app::HintToken;

    #[test]
    fn groups_input_hint_approximations_after_all_primary_tokens() {
        let spans = input_hint_spans(&[
            HintToken {
                primary: "1/10".to_string(),
                approximation: Some("0.1f".to_string()),
            },
            HintToken {
                primary: "0.2f".to_string(),
                approximation: None,
            },
        ]);

        let rendered = spans
            .into_iter()
            .map(|span| span.content.into_owned())
            .collect::<String>();

        assert_eq!(rendered, "1/10 0.2f | 0.1f 0.2f");
    }
}
