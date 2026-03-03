# lifocalc

`lifocalc` is a terminal RPN calculator built with Rust and ratatui.

## Current boilerplate

- Bottom input line and top stack area.
- Stack is bottom-aligned and reverse-numbered (`1:` is the last visible line).
- `Enter` behavior:
  - numbers-only input pushes values to the global stack and clears input.
  - operator-only input consumes values from the global stack, places the result in input, and leaves the stack without that result.
  - expressions that include inline numbers are evaluated against a prompt-local stack seeded from the global stack, then input is replaced with the result.
- History navigation with up/down arrows.
- Live hint shown after the current input.
- Inline status message for evaluation errors.

## Project layout

- `src/engine/mod.rs`: parser and math evaluation logic.
- `src/app.rs`: app state transitions (input, stack, history, status).
- `src/ui.rs`: ratatui rendering.
- `src/main.rs`: terminal setup and event loop.

## YAML test framework

Test cases live in `tests/cases/*.yaml` and are executed by `tests/yaml_cases.rs`.
Use numbered filenames (`0001.yaml`, `0002.yaml`, ...) and describe each case inside YAML.

Each file defines one interaction step:

```yaml
description: "evaluates multiplication from stack values"
before_stack: ["3", "4"]
input: "*"
expected_after_stack: ["3", "4"]
expected_after_input: "12"
```

Optional:

```yaml
expected_status: "division by zero"
```

### Why strings for numbers?

All numeric values in YAML are strings to keep test files stable while the internal numeric representation evolves (for example from `f64` to a custom numeric type).

### Add a new test

1. Create a new `.yaml` file under `tests/cases/`.
2. Fill in the required fields (`description`, `before_stack`, `input`, `expected_after_stack`, `expected_after_input`).
3. Run `cargo test`.
