use crate::engine;

#[derive(Debug, Default)]
pub struct App {
    input: String,
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
        self.history_index = None;
        self.status = None;
    }

    pub fn insert_char(&mut self, character: char) {
        self.input.push(character);
        self.history_index = None;
        self.status = None;
    }

    pub fn backspace(&mut self) {
        self.input.pop();
        self.history_index = None;
        self.status = None;
    }

    pub fn clear_input(&mut self) {
        self.input.clear();
        self.history_index = None;
        self.status = None;
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
            engine::evaluate_expression(&entry, &self.stack)
        };

        match result {
            Ok(value) => {
                if should_mutate_global_stack {
                    self.stack = candidate_stack;
                    self.input.clear();
                } else {
                    self.stack.push(value);
                    self.input.clear();
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
        self.status = None;
    }

    pub fn history_down(&mut self) {
        let Some(index) = self.history_index else {
            return;
        };

        if index + 1 >= self.history.len() {
            self.history_index = None;
            self.input.clear();
            self.status = None;
            return;
        }

        let next_index = index + 1;
        self.history_index = Some(next_index);
        self.input = self.history[next_index].clone();
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

        engine::evaluate_expression(trimmed, &self.stack)
            .ok()
            .map(engine::format_number)
    }

    pub fn stack_as_strings(&self) -> Vec<String> {
        self.stack
            .iter()
            .map(|value| engine::format_number(*value))
            .collect()
    }
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
}
