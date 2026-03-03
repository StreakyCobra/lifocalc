use crate::engine;

#[derive(Debug, Default)]
pub struct App {
    input: String,
    cursor: usize,
    stack: Vec<f64>,
    history: Vec<String>,
    history_index: Option<usize>,
    status: Option<String>,
}

impl App {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn input(&self) -> &str {
        &self.input
    }

    pub fn cursor(&self) -> usize {
        self.cursor
    }

    pub fn stack(&self) -> &[f64] {
        &self.stack
    }

    pub fn set_stack(&mut self, stack: Vec<f64>) {
        self.stack = stack;
    }

    pub fn status(&self) -> Option<&str> {
        self.status.as_deref()
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

    pub fn submit_input(&mut self) {
        let trimmed = self.input.trim();
        if trimmed.is_empty() {
            self.status = None;
            return;
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

            return;
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

    pub fn hint(&self) -> Option<String> {
        let trimmed = self.input.trim();
        if trimmed.is_empty() {
            return None;
        }

        if engine::is_numbers_only(trimmed) {
            return Some(trimmed.to_string());
        }

        if engine::has_number_token(trimmed) {
            engine::evaluate_expression(trimmed, &[])
                .ok()
                .map(engine::format_number)
        } else {
            engine::evaluate_expression(trimmed, &self.stack)
                .ok()
                .map(engine::format_number)
        }
    }

    pub fn stack_as_strings(&self) -> Vec<String> {
        self.stack
            .iter()
            .map(|value| engine::format_number(*value))
            .collect()
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
    use super::App;

    #[test]
    fn history_navigation_round_trips_to_empty_input() {
        let mut app = App::new();
        app.set_input("2");
        app.submit_input();
        app.set_input("3 4 +");
        app.submit_input();

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
}
