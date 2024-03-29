# Kubekeeper Changelog

## Unreleased

- Mirror repository to gitlab
- Bump markdownlint-cli2 action from 8 to 10 (<https://github.com/badouralix/kubekeeper/pull/7>)

## v2.5.0

- Apply clippy lint suggestions on macro rules
- Bump markdownlint-cli2 action from 7 to 8 (<https://github.com/badouralix/kubekeeper/pull/3>)
- Update magic to detect native kubectl commands (<https://github.com/badouralix/kubekeeper/pull/5>)
- Fix link in changelog

## v2.4.0

- Use official markdownlint github action
- Use variable debug format in debug logs
- Update command include and exclude lists
- Print included contexts in red

## v2.3.0

- Rewrite table-driven tests
- Display namespace in validation prompt
- Add new check context empty test
- Add rustfmt config file
- Add plain text reason when identifying actions
- Print kubekeeper output to stderr
- Update doc comments
- Add debug mode

## v2.2.0

- Add clippy check
- Simplify save context return value
- Highlight current context in bold yellow
- Rename clippy job
- Apply clippy lint suggestions automagically
- Fix remaining clippy warnings
- Add changelog
- Add support for patterns in context include and exclude lists
- Run cargo test in github actions
- Update context include and exclude lists
- Update demo screenshot with new validation prompt

## v2.1.0

- Add doc link to cobra command to request completion
- Return non-zero exit code when validation failed
- Prefix current context to native kubectl commands only
- Validate context without having to press enter
- Fix double newline on user input

## v2.0.2

- Remove debug logs
- Include commands edit and label
- Fix cobra dynamic completion

## v2.0.1

- Add demo screenshot
- Add lint workflow
- Add dependabot config file
- Bump actions/checkout from 2 to 3 (<https://github.com/badouralix/kubekeeper/pull/2>)
- Add new commands without validation

## v2.0.0

- Rewrite kubekeeper in rust
- Add cargo lock
- Add new installation and configuration steps
- Bump license year
- Only append context if it is not already set

## v1.3.0

- Use logging instead of print
- Lint python files
- Add debug logs
- Exclude cobra completion hidden function

## v1.2.0

- Exclude help commands
- Prevent run with `--cluster`

## v1.1.0

- Increase fzf height
- Add default config
- Add uninstall command to bootstrap
- Include commands apply and scale
- Add doc for autocompletion
- Fix curl-sh install
- Add missing ro commands to the exclude list
- Fix context save heuristic
- Use kube dir for default config dir
- Troubleshoot kubectl-fzf completion

## v1.0.0

- Commit for a dream
