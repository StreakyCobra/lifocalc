use crate::{config::DisplayConfig, engine};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HintToken {
    pub primary: String,
    pub approximation: Option<String>,
}

#[derive(Debug, Default)]
pub struct App {
    input: String,
    cursor: usize,
    stack: Vec<engine::Number>,
    history: Vec<String>,
    history_index: Option<usize>,
    status: Option<String>,
    display_config: DisplayConfig,
}

impl App {
    pub fn new() -> Self {
        Self::new_with_display_config(DisplayConfig::default())
    }

    pub fn new_with_display_config(display_config: DisplayConfig) -> Self {
        Self {
            display_config,
            ..Self::default()
        }
    }

    pub fn input(&self) -> &str {
        &self.input
    }

    pub fn cursor(&self) -> usize {
        self.cursor
    }

    pub fn stack(&self) -> &[engine::Number] {
        &self.stack
    }

    pub fn set_stack(&mut self, stack: Vec<engine::Number>) {
        self.stack = stack;
    }

    pub fn set_history(&mut self, history: Vec<String>) {
        self.history = history;
        self.history_index = None;
    }

    pub fn status(&self) -> Option<&str> {
        self.status.as_deref()
    }

    pub fn display_config(&self) -> DisplayConfig {
        self.display_config
    }

    pub fn set_input(&mut self, input: impl Into<String>) {
        self.input = input.into();
        self.cursor = self.input.chars().count();
        self.history_index = None;
        self.status = None;
    }

    pub fn insert_char(&mut self, character: char) {
        let insert_at = byte_index_for_char(&self.input, self.cursor);
        self.input.insert(insert_at, character);
        self.cursor += 1;
        self.history_index = None;
        self.status = None;
    }

    pub fn backspace(&mut self) {
        if self.cursor == 0 {
            return;
        }

        self.remove_char_before_cursor();
        self.history_index = None;
        self.status = None;
    }

    pub fn clear_input(&mut self) {
        self.input.clear();
        self.cursor = 0;
        self.history_index = None;
        self.status = None;
    }

    pub fn move_cursor_left(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
        }
    }

    pub fn move_cursor_right(&mut self) {
        let len = self.input.chars().count();
        if self.cursor < len {
            self.cursor += 1;
        }
    }

    pub fn delete_word_backward(&mut self) {
        self.history_index = None;
        self.status = None;

        while self
            .char_before_cursor()
            .is_some_and(|character| character.is_whitespace())
        {
            self.remove_char_before_cursor();
        }

        while self
            .char_before_cursor()
            .is_some_and(|character| !character.is_whitespace())
        {
            self.remove_char_before_cursor();
        }

        while self
            .char_before_cursor()
            .is_some_and(|character| character.is_whitespace())
            && self
                .char_at_cursor()
                .is_some_and(|character| character.is_whitespace())
        {
            self.remove_char_before_cursor();
        }
    }

    pub fn submit_input(&mut self) -> Option<String> {
        let trimmed = self.input.trim();
        if trimmed.is_empty() {
            self.status = None;
            return None;
        }

        let entry = trimmed.to_string();
        self.history.push(entry.clone());
        self.history_index = None;

        if engine::is_numbers_only(&entry) {
            match engine::parse_numbers(&entry) {
                Ok(numbers) => {
                    self.stack.extend(numbers);
                    self.input.clear();
                    self.cursor = 0;
                    self.status = None;
                }
                Err(error) => {
                    self.status = Some(error.to_string());
                }
            }

            return Some(entry);
        }

        let should_mutate_global_stack = !engine::has_number_token(&entry);

        let mut candidate_stack = self.stack.clone();
        let result = if should_mutate_global_stack {
            engine::evaluate_expression_in_place(&entry, &mut candidate_stack)
        } else {
            engine::evaluate_expression(&entry, &[])
        };

        match result {
            Ok(value) => {
                if should_mutate_global_stack {
                    self.stack = candidate_stack;
                    self.input.clear();
                    self.cursor = 0;
                } else {
                    self.stack.push(value);
                    self.input.clear();
                    self.cursor = 0;
                }
                self.status = None;
            }
            Err(error) => {
                self.status = Some(error.to_string());
            }
        }

        Some(entry)
    }

    pub fn history_up(&mut self) {
        if self.history.is_empty() {
            return;
        }

        let next_index = match self.history_index {
            Some(index) if index > 0 => index - 1,
            Some(_) => 0,
            None => self.history.len().saturating_sub(1),
        };

        self.history_index = Some(next_index);
        self.input = self.history[next_index].clone();
        self.cursor = self.input.chars().count();
        self.status = None;
    }

    pub fn history_down(&mut self) {
        let Some(index) = self.history_index else {
            return;
        };

        if index + 1 >= self.history.len() {
            self.history_index = None;
            self.input.clear();
            self.cursor = 0;
            self.status = None;
            return;
        }

        let next_index = index + 1;
        self.history_index = Some(next_index);
        self.input = self.history[next_index].clone();
        self.cursor = self.input.chars().count();
        self.status = None;
    }

    pub fn hint(&self) -> Option<Vec<HintToken>> {
        let trimmed = self.input.trim();
        if trimmed.is_empty() {
            return None;
        }

        if engine::is_numbers_only(trimmed) {
            return engine::parse_numbers(trimmed).ok().map(|numbers| {
                numbers
                    .iter()
                    .map(|number| self.hint_token(number))
                    .collect()
            });
        }

        if engine::has_number_token(trimmed) {
            engine::evaluate_expression(trimmed, &[])
                .ok()
                .map(|number| vec![self.hint_token(&number)])
        } else {
            engine::evaluate_expression(trimmed, &self.stack)
                .ok()
                .map(|number| vec![self.hint_token(&number)])
        }
    }

    pub fn stack_as_strings(&self) -> Vec<String> {
        self.stack
            .iter()
            .map(engine::format_number)
            .collect()
    }

    fn hint_token(&self, number: &engine::Number) -> HintToken {
        let formatted = engine::format_number_parts(number);

        HintToken {
            primary: formatted.primary,
            approximation: if self.display_config.approximation_hint.input {
                formatted.approximation
            } else {
                None
            },
        }
    }

    fn char_before_cursor(&self) -> Option<char> {
        if self.cursor == 0 {
            return None;
        }

        self.input.chars().nth(self.cursor - 1)
    }

    fn char_at_cursor(&self) -> Option<char> {
        self.input.chars().nth(self.cursor)
    }

    fn remove_char_before_cursor(&mut self) {
        if self.cursor == 0 {
            return;
        }

        let start = byte_index_for_char(&self.input, self.cursor - 1);
        let end = byte_index_for_char(&self.input, self.cursor);
        self.input.replace_range(start..end, "");
        self.cursor -= 1;
    }
}

fn byte_index_for_char(input: &str, char_index: usize) -> usize {
    if char_index == 0 {
        return 0;
    }

    input
        .char_indices()
        .nth(char_index)
        .map(|(index, _)| index)
        .unwrap_or(input.len())
}

#[cfg(test)]
mod tests {
    use super::{App, HintToken};
    use crate::{
        config::{ApproximationHintConfig, DisplayConfig},
        engine,
    };

    #[test]
    fn history_navigation_round_trips_to_empty_input() {
        let mut app = App::new();
        app.set_input("2");
        let _ = app.submit_input();
        app.set_input("3 4 +");
        let _ = app.submit_input();

        app.history_up();
        assert_eq!(app.input(), "3 4 +");
        app.history_up();
        assert_eq!(app.input(), "2");
        app.history_down();
        assert_eq!(app.input(), "3 4 +");
        app.history_down();
        assert_eq!(app.input(), "");
    }

    #[test]
    fn delete_word_backward_removes_last_word() {
        let mut app = App::new();
        app.set_input("12 34  ");

        app.delete_word_backward();

        assert_eq!(app.input(), "12 ");
    }

    #[test]
    fn delete_word_backward_respects_cursor_position() {
        let mut app = App::new();
        app.set_input("12 34 56");
        app.move_cursor_left();
        app.move_cursor_left();
        app.move_cursor_left();

        app.delete_word_backward();

        assert_eq!(app.input(), "12 56");
    }

    #[test]
    fn cursor_left_right_navigates_input() {
        let mut app = App::new();
        app.set_input("12");
        app.move_cursor_left();
        app.insert_char('x');

        assert_eq!(app.input(), "1x2");
    }

    #[test]
    fn numbers_only_hint_uses_exact_formatting() {
        let mut app = App::new();
        app.set_input("0.125 10/6");

        assert_eq!(
            app.hint(),
            Some(vec![
                HintToken {
                    primary: "1/8".to_string(),
                    approximation: Some("0.125f".to_string()),
                },
                HintToken {
                    primary: "5/3".to_string(),
                    approximation: Some("1.6666666666666667f".to_string()),
                },
            ])
        );
    }

    #[test]
    fn operator_hint_uses_exact_fraction_result() {
        let mut app = App::new();
        app.set_stack(vec![
            engine::parse_number("1").expect("expected valid number"),
            engine::parse_number("3").expect("expected valid number"),
        ]);
        app.set_input("/");

        assert_eq!(
            app.hint(),
            Some(vec![HintToken {
                primary: "1/3".to_string(),
                approximation: Some("0.3333333333333333f".to_string()),
            }])
        );
    }

    #[test]
    fn numbers_only_hint_preserves_approximate_literals() {
        let mut app = App::new();
        app.set_input("1 0.5f");

        assert_eq!(
            app.hint(),
            Some(vec![
                HintToken {
                    primary: "1".to_string(),
                    approximation: Some("1f".to_string()),
                },
                HintToken {
                    primary: "0.5f".to_string(),
                    approximation: None,
                },
            ])
        );
    }

    #[test]
    fn operator_hint_shows_approximate_result() {
        let mut app = App::new();
        app.set_stack(vec![engine::parse_number("2").expect("expected valid number")]);
        app.set_input("sqrt");

        assert_eq!(
            app.hint(),
            Some(vec![HintToken {
                primary: "1.4142135623730951f".to_string(),
                approximation: None,
            }])
        );
    }

    #[test]
    fn input_hint_can_disable_exact_approximations() {
        let mut app = App::new_with_display_config(DisplayConfig {
            approximation_hint: ApproximationHintConfig {
                stack: true,
                input: false,
            },
        });
        app.set_input("1/2");

        assert_eq!(
            app.hint(),
            Some(vec![HintToken {
                primary: "1/2".to_string(),
                approximation: None,
            }])
        );
    }
}
