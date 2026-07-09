# openclaudecode

This npm package installs the `occ`, `openclaude`, and `openclaudecode` launchers for openclaude.

openclaude is a local Claude Code gateway proxy. It starts a local `occ` daemon, injects
`ANTHROPIC_BASE_URL` / `ANTHROPIC_API_KEY` for Claude Code, and routes Claude Messages API traffic to
configured providers.

## Install

```bash
npm install -g openclaudecode
occ init
claude
```

For source checkouts:

```bash
cargo build --release
npm install -g ./npm
```

See the repository README for provider setup, native-mode behavior, security notes, and development
commands.
