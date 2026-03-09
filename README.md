# lifocalc

`lifocalc` is a terminal RPN calculator built with Rust and ratatui.

## Current boilerplate

- Bottom input line and top stack area.
- Stack is bottom-aligned and reverse-numbered (`1:` is the last visible line).
- `Enter` behavior:
  - numbers-only input pushes values to the global stack and clears input.
  - operator-only input consumes values from the global stack and pushes the result back to the stack.
  - expressions that include inline numbers are evaluated against an independent prompt-local stack, then the full prompt-local result stack is appended to the global stack.
- History navigation with up/down arrows.
- Input history is persisted across app restarts in `${XDG_STATE_HOME}/lifocalc/history`, or `~/.local/state/lifocalc/history` when `XDG_STATE_HOME` is unset.
- Persisted history entries are de-duplicated, and expressions that fail evaluation are not written to the history file.
- Live hint shown after the current input.
- Exact values can show a gray approximate `| ...f` suffix in the stack and live input hint.
- Inline status message for evaluation errors.
- Supported operators in this boilerplate: `+`, `-`, `*`, `/`, `sum`.
- Numeric values support exact and approximate modes:
  - exact inputs like `0.125`, `1e3`, and `10/6` are rendered canonically as `1/8`, `1000`, and `5/3`
  - approximate inputs use an `f` suffix, such as `0.5f` or `1e-3f`
  - `~` converts an exact value to approximate, for example `1 2 / ~` -> `0.5f`
  - `sqrt`, `ln`, `exp`, `sin`, `cos`, and `tan` run in approximate mode and return `f`-suffixed values
- Quantities can carry units:
  - quantity literals use `number[unit]`, such as `1[kB]`, `60[s]`, or `1[MB/s]`
  - unitless values still work exactly as before, for example `1 2 +` -> `3`
  - values are stored in canonical base units internally and auto-formatted into a readable unit in the range `[1, 1000)` when possible
  - explicit conversion uses `in`, for example `1[MB/s] [kB/s] in`
  - implicit conversion shorthand is also supported, so `1[MB/s] [kB/s] 2 *` behaves like `1[MB/s] [kB/s] in 2 *`
  - mixed-unit `+` and `-` require compatible dimensions, while `*` and `/` can produce derived units like `kB/s`
  - the initial unit registry includes data (`b`, `B`) and time (`s`, `min`, `h`, `d`) with SI prefixes such as `k`, `M`, `m`, and `u`

## Project layout

- `src/engine/mod.rs`: parser and math evaluation logic.
- `src/app.rs`: app state transitions (input, stack, history, status).
- `src/ui.rs`: ratatui rendering.
- `src/main.rs`: terminal setup and event loop.

The engine dispatches operators/functions through a registry with explicit arity (`Exact(n)` or `AtLeast(n)`), so adding new functions is mostly adding one function definition entry and one evaluator function.

## YAML test framework

Test cases live in `tests/cases/*.yaml` and are executed by `tests/yaml_cases.rs`.
Use numbered filenames (`0001.yaml`, `0002.yaml`, ...) and describe each case inside YAML.
These YAML files are the primary behavior tests; Rust unit tests are kept for lower-level internals such as engine helpers and history navigation.

Each file defines one interaction step:

```yaml
description: "evaluates multiplication from stack values"
before_stack: ["3", "4"]
input: "*"
expected_after_stack: ["12"]
expected_after_input: ""
```

Optional:

```yaml
expected_status: "division by zero"
```

```yaml
expected_before_hint:
  - primary: "1/3"
    approximation: "0.3333333333333333f"
```

### Why strings for numbers?

All numeric values in YAML are strings to keep test files stable while the internal numeric representation evolves (for example from `f64` to a custom numeric type).

### Add a new test

1. Create a new `.yaml` file under `tests/cases/`.
2. Fill in the required fields (`description`, `before_stack`, `input`, `expected_after_stack`, `expected_after_input`).
3. Add `expected_before_hint` when you need to assert the live hint before pressing `Enter`.
4. Run `cargo test`.

## Configuration

`lifocalc` uses an embedded TOML default config from `config/default-config.toml` as the source of truth.

- User overrides are loaded from `${XDG_CONFIG_HOME}/lifocalc/config.toml`, or `~/.config/lifocalc/config.toml` when `XDG_CONFIG_HOME` is unset.
- Final display settings and keymap are `embedded defaults + user overrides`.
- Unit conversion shorthand can be disabled with `[units].implicit_conversion = false`, which requires explicit `in` after a bare unit spec.
- Use `none` to disable a default binding.
- Invalid keys or unknown action IDs are ignored with a warning.

Example:

```toml
[display.approximation_hint]
stack = true
input = false

[units]
implicit_conversion = true

[keybindings]
"pageup" = "history.prev"
"pagedown" = "history.next"
"esc" = "app.exit"
"ctrl+c" = "app.exit"
"ctrl+l" = "app.clear_input"
"ctrl+backspace" = "app.delete_word_backward"
"ctrl+h" = "app.delete_word_backward"
"ctrl+w" = "app.delete_word_backward"
"left" = "app.cursor_left"
"right" = "app.cursor_right"
"up" = "none"
"down" = "none"
```

Built-in action IDs:

- `app.exit`
- `app.evaluate`
- `app.backspace`
- `app.delete_word_backward`
- `app.cursor_left`
- `app.cursor_right`
- `history.prev`
- `history.next`
- `app.clear_input`
- `none` (special unbind value)
