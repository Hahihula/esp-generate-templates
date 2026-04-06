# esp-generate-templates

Alternative templates for [esp-generate](https://github.com/esp-rs/esp-generate) inspired by the BSP concept from esp-idf.

## Overview

This repository contains board support templates for esp-generate, designed to provide a similar experience to esp-idf's BSP (Board Support Package) system. Each template is a complete project scaffold tailored for specific hardware.

## Current Template

### ESP32-C6 Touch LCD 1.47" (Waveshare)

Located in `esp32c6-touch-lcd-template/`, this template is for the [Waveshare ESP32-C6-Touch-LCD-1.47](https://www.waveshare.com/esp32-c6-touch-lcd-1.47.htm) board.

Features:
- Dysplay
- Touch
- IMU
- Ferris bitmap demo for display testing

## Usage

**Note:** This feature requires a custom fork of esp-generate. Currently only [this branch](https://github.com/Hahihula/esp-generate/tree/add-support-for-custom-templates) supports custom templates via the `--bsp` parameter.

1. Download this repository
2. Run esp-generate with the `--bsp` parameter pointing to the desired template:

```bash
esp-generate --bsp /path/to/esp-generate-templates/esp32c6-touch-lcd-template
```