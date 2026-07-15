<div align="center">
  <h1><code>$ ssh eipi.boo</code></h1>
  <p>🫶🏼 confess over ssh.</p>
</div>


`eipi.boo`: An ssh app where you write confessions and they float around on a shared canvas. everyone sees the same world. you can react, reply, search, change themes, all from your terminal.

built with [russh](https://github.com/Eugeny/russh) + [ratatui](https://github.com/ratatui/ratatui) in rust.

## Pronunciation

**eipi.boo**

> /ˌiː piː ˈbuː/  
> **ee PEE BOO**

## keybinds

| key | what it does |
|-----|-------------|
| `h/j/k/l` or arrows | move around the canvas |
| `n` | write a confession |
| `space` | card view (flip through confessions) |
| `tab` | cycle selection |
| `enter` | view replies |
| `r` | reply to a confession |
| `f` | react |
| `/` | search |
| `T` | change theme |
| `?` | help |
| `q` | quit |

## themes

17 themes built in. press `T` to pick one. it persists across sessions and changes your whole terminal colors (background, foreground, everything).

rose pine, rose pine moon, rose pine dawn, catppuccin, catppuccin latte, catppuccin frappe, catppuccin macchiato, dracula, gruvbox, nord, tokyo night, kanagawa, everforest, one dark, monokai, solarized, and the default.

adding a theme is just dropping a file in `src/tui/themes/` and registering it in `mod.rs`.

## run your own

```
git clone https://github.com/pwnwriter/eipi.boo
cargo build --release
EIPI_LISTEN=0.0.0.0:2222 ./target/release/eipi
```

or with nix:
```
nix develop
cargo run
ssh localhost -p 2222
```

there's a [justfile](justfile) too: `just build`, `just deploy`, `just fmt`, etc.

## how it works

you ssh in, the server gives you a TUI. confessions are stored in sqlite. your ssh fingerprint is hashed (sha256 of sha256) and used for rate limiting and reaction dedup. it's never displayed. no accounts, no passwords, no cookies.

the canvas is infinite. confessions get placed randomly with spacing so they don't overlap. popular ones glow. reactions float around them.

<details>
<summary><strong>security & privacy</strong></summary>

**what the server sees:**
- your ssh public key fingerprint (hashed): used for rate limiting, never displayed
- your ip: visible in connection logs like any server, not stored in the db
- your confessions, reactions, and replies: stored with only the hashed fingerprint

**what it can't do:**
- access your files, shell, or anything on your machine
- read your private ssh key
- forward your ssh agent or x11

</details>

## contributing

prs welcome. [open an issue](https://github.com/pwnwriter/eipi.boo/issues/new) or just send a pr.

## license

[MIT](LICENSE)

<p align="center"><img src="https://raw.githubusercontent.com/catppuccin/catppuccin/main/assets/footers/gray0_ctp_on_line.svg?sanitize=true" /></p>
<p align="center">Copyright &copy; 2026 - present <a href="https://pwnwriter.me" target="_blank">pwnwriter</a></p>
