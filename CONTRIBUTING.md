# Contributing to Mantle

First off, thanks for considering a contribution! Mantle is a small project and every bit of help - code, docs, bug reports, or just asking a question - is genuinely welcome. Don't worry about your PR or issue being "too small" or "not good enough"; if in doubt, open it anyway and we'll figure it out together.

## Getting a dev environment running

1. Clone the repo and copy `config.example.toml` to `config.toml`, filling in the required fields (see the Setup section in the README).
2. Start the backing services:
   ```bash
   docker compose -f docker-compose.dev.yml up -d
   ```
   This brings up Postgres on 5432.
3. Run the app:
   ```bash
   cargo run --bin mantle
   ```
   Migrations run automatically on startup - no separate migrate step needed.

That's it - no need to containerize the app itself while developing, `cargo run` is faster to iterate on.

## Project layout

- `src/main.rs` - the main service
- Other `--bin` targets exist for auxiliary tools (e.g. `generate-openapi`) - run with `cargo run --bin <name>`
- `migrations/` — sqlx migrations, applied automatically at startup

## Making a change

- Fork the repo, then branch off with a `<type>/<name>` naming pattern (e.g. `fix/discord-cache-race`, `feat/boosty-linking`). Not strictly enforced, but it makes the branch list a lot easier to scan.
- Commit messages ideally follow [Conventional Commits](https://www.conventionalcommits.org/) (`feat: ...`, `fix: ...`, `chore: ...`, etc.) - nice to have, not a blocker. Same goes for the branch naming above: do it if convenient, don't sweat it if not.
- Keep PRs focused - one logical change per PR is much easier to review than a grab-bag.
- Before pushing:
  ```bash
  cargo fmt
  cargo clippy --all --all-features
  ```
- If you're touching the database schema, add a migration:
  ```bash
  cargo install sqlx-cli --no-default-features --features postgres  # if you don't have it yet
  sqlx migrate add <short_description>
  ```
  Don't edit existing migration files that have already been merged - add a new one instead.

## Tests

```bash
cargo test
```
If you're adding non-trivial logic, a test is appreciated but not a hard blocker - happy to help figure out how to test something tricky in review.

## Opening a PR

- Use the PR template - it's there to make sure nothing important gets missed, please fill it in rather than deleting it.
- Briefly describe **what** changed and **why**.
- Link the related issue if there is one.
- It's fine to open a PR early and mark it as a draft if you want feedback on direction before finishing.

## Reporting bugs / suggesting features

Open an issue using the template that matches what you're reporting (bug report, feature request, etc.) — each one asks for slightly different info that helps us act on it faster. A quick search first to avoid duplicates is appreciated, but don't stress over it - worst case I'll close it as a dupe and point you to the original.

## Questions

Not sure about something - architecture, whether a feature fits, how to approach a bug? Open an issue for it (there's no Discord or chat, issues are the one place for everything) rather than sitting on it. Questions are welcome, not an interruption.

## License

Mantle is licensed under MIT - see [`LICENSE.TXT`](./LICENSE.TXT). By contributing, you agree your contributions are licensed under the same terms.
