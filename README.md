# esp-generate-templates

Alternative templates for [esp-generate](https://github.com/esp-rs/esp-generate), inspired by the BSP concept from esp-idf.

## Overview

This repository contains board support templates for esp-generate, designed to provide a similar experience to esp-idf's BSP (Board Support Package) system. Each template is a complete project scaffold tailored for specific hardware.

## Requirements

External template support is upstream in esp-generate — the custom fork and its `--bsp` flag are no longer needed. It landed after the v1.4.0 release, though, so until the next release you need a build from `main`:

```bash
cargo install --git https://github.com/esp-rs/esp-generate --locked
```

## Usage

Point `--template` at this repository and esp-generate clones it and finds the template inside:

```bash
esp-generate --template Hahihula/esp-generate-templates my-project
```

Or clone it yourself and point at a directory:

```bash
git clone https://github.com/Hahihula/esp-generate-templates
esp-generate --template esp-generate-templates/esp32c6-touch-lcd-template my-project
```

`--template` accepts a local directory, `owner/repo`, `owner/repo@branch-or-tag`, an `https://` URL, or `git@host:path`. Repositories are cloned shallowly into a temporary directory that is deleted when esp-generate exits, using your existing git credentials — so private template repositories work too. The resolved commit is logged, so a generated project can be traced back to exactly what produced it.

> The bare `owner/repo` form works because this repository currently holds exactly one template. Once there is more than one, esp-generate will refuse to guess and list them instead; point `--template` at the directory you want.

Generating from an external template prints a warning, since a template decides what code and dependencies end up in your project. Only use templates you trust.

### Inspecting a template

`list-options` and `explain` take the same `--template`:

```bash
esp-generate --template Hahihula/esp-generate-templates list-options
esp-generate --template Hahihula/esp-generate-templates explain touch
```

### Headless generation

```bash
esp-generate --template esp-generate-templates/esp32c6-touch-lcd-template --headless \
  -o esp32c6 -o esp32-c6-touch-lcd-1_47 \
  -o alloc -o unstable-hal -o embassy \
  -o display -o touch -o imu -o temp-sensor -o demo \
  my-project
```

## Templates

### ESP32-C6 Touch LCD 1.47" (Waveshare)

In `esp32c6-touch-lcd-template/`, for the [Waveshare ESP32-C6-Touch-LCD-1.47](https://www.waveshare.com/esp32-c6-touch-lcd-1.47.htm) board.

Board options:

- `display` — LCD support (requires `embassy`)
- `touch` — AXS5106L touch controller (requires `display`)
- `imu` — QMI8658 IMU (requires `embassy`)
- `temp-sensor` — on-chip temperature sensor
- `demo` — demo application tying the above together, including a Ferris bitmap for display testing

Alongside the usual esp-generate options: `alloc`, `unstable-hal`, `embassy`, `wifi`, BLE (`ble-bleps` / `ble-trouble`), `defmt` / `log`, `probe-rs`, `embedded-test`, `wokwi`, `ci`, and editor settings for Helix, Neovim, VS Code and Zed.

## Writing a template

A template is a directory with two files at its root:

- **`metadata.toml`** — the manifest. Declares the `esp-template-sdk` version the template targets, the plugins it needs (`plugins = { chip = "0.5.0" }` for the `chip.*` facts), and `emit` rules saying which files are emitted, under what condition (`when`) and at what path (`as`). A file with no rule is emitted as-is.
- **`template.yaml`** — the option tree: the options, their `selection_group`, `requires`, and `compatible` constraints.

Anything under `.template/` is machinery — option-tree fragments and `#%include` partials — and is never emitted.

File bodies are processed with [somni-template](https://docs.rs/somni-template) directives, marked with the file's own comment prefix plus `%` so ordinary comments are never parsed as directives:

```rust
//%if option("display")
//+    let display = Display::new(spi, dc, rst)?;
//%endif
```

and `{{ }}` for interpolation — `{{ project_name }}`, `{{ chip.name }}`, `{{ chip.rust_target }}`.

Before pushing a change, sweep the template across its option combinations without writing anything:

```bash
esp-generate --template ./esp32c6-touch-lcd-template check
```

This catches mistakes a single generation cannot — a typo in a branch that one selection never evaluates. Add `--build` to also compile each combination.
