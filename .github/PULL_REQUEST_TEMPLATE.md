## Summary

- 

## Validation

- [ ] `cargo fmt --check`
- [ ] `cargo test`
- [ ] `cargo clippy --all-targets -- -D warnings`
- [ ] `npm pack --dry-run`
- [ ] Manual Claude Code / provider smoke test, if relevant

## Security checklist

- [ ] This change does not log provider API keys, gateway tokens, OAuth tokens, or full config files.
- [ ] New files that may contain credentials are written with owner-only permissions where supported.
- [ ] Claude Code subscription OAuth remains native-only and is not extracted or proxied.

## Compatibility

- [ ] Command behavior remains compatible with the documented `occ` surface.
- [ ] Any intentional behavior change is documented in `README.md` or `CHANGELOG.md`.
