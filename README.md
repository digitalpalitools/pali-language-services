[![Continuous Deployment](https://github.com/digitalpalitools/pali-language-services/workflows/Continuous%20Deployment/badge.svg)](https://github.com/digitalpalitools/lib/actions?query=workflow%3A%22Continuous+Deployment%22) [![License: CC BY-NC-SA 4.0](https://img.shields.io/badge/License-CC%20BY--NC--SA%204.0-lightgrey.svg)](https://creativecommons.org/licenses/by-nc-sa/4.0/) [![npm latest](https://img.shields.io/npm/v/@digitalpalitools/pali-language-services/latest.svg)](https://www.npmjs.com/package/@digitalpalitools/pali-language-services)

# Pāli Language Services

## Purpose

Define the basics of the Pāli language in code as per the [specifications](http://bit.ly/dptvision).

This is a cross platform library usable both on frontend and backend.

## Features

- Pāli alphabet
  - [x] Roman script
  - [x] Parsing pāli written in Roman script
  - [x] Compare order for strings
- Inflections
  - [x] Generate inflection tables
  - [ ] Generate all inflected words
- [x] Publish as npm library
- [ ] Publish on crates.io

## Using PLS

- ```yarn add @digitalpalitools/pali-language-services --force```
- ```import * as PSL from '@digitalpalitools/pali-language-services'```

## Updating PLS

- ```cargo clean; cargo build; wasm-pack build --scope digitalpalitools --target bundler```
- ```npm login # dptadmin / digitalpalitools@gmail.com / <passsowrd>```
- ```wasm-pack publish --access public```

## Commands

- Build: ```cargo clean; cargo build; wasm-pack build --scope digitalpalitools --target bundler```
- Test: ```cargo test```
- Format: ```cargo clean; cargo fmt --all -- --check```
- Clippy: ```cargo clean; cargo clippy --tests --all-targets --all-features -- -D warnings```
- Watch Tests: ```cargo watch -x test```
