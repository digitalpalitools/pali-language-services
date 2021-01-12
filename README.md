[![Continuous Deployment](https://github.com/digitalpalitools/lib/workflows/Continuous%20Deployment/badge.svg)](https://github.com/digitalpalitools/lib/actions?query=workflow%3A%22Continuous+Deployment%22)

# Digital Pāli Tools - Core Library

## Purpose

Define the basics of the Pāli language in code as per the [specifications](https://docs.google.com/document/d/1KF6NLFiiVH9oVz_NcU5mjHcMcIAZECgNifM8mX25MCo/edit#heading=h.2hvqs8bpra4).

This is a cross platform library usable both on frontend and backend.

## Features

- [ ] Consumable as WebAsm.
- [x] Pāli alphabet
- [x] Pāli alphabet - Roman script
  - [x] Parsing pāli written in Roman script
  - [x] Compare order for strings
- [ ] Publish as npm library

## Commands

- Build: ```cargo build```
- Test: ```cargo test```
- Format: ```cargo fmt --all -- --check```
- Clippy: ```cargo clippy --tests -- -D warnings```
- Watch Tests: ```cargo watch -x test```
