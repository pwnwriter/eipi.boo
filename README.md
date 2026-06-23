<div align="center">
  <h1><code>$ ssh eipi.boo</code></h1> 
</div>

<details>
<summary><strong>Security & Privacy</strong></summary>

**What "anonymous" means here:** Other users can't see who posted what. There are no usernames, no accounts, no profiles. Your confessions, votes, and replies are not tied to any visible identity.

**What the server can see:**
- Your SSH public key fingerprint (SHA-256) — used for rate limiting and vote deduplication, not displayed anywhere
- Your IP address — visible in connection logs like any server, not stored in the database
- Your confessions, votes, and replies — stored in SQLite with only the fingerprint as author identifier

**What the server cannot do:**
- Access your files, shell, or anything on your machine
- Read your private SSH key
- Forward your SSH agent or X11 (these are off by default, you'd have to explicitly pass `-A` or `-X`)

**If you don't trust the live server**, clone the repo and run your own:
```
git clone https://github.com/pwnwriter/eipi.boo
cargo build --release
EIPI_LISTEN=0.0.0.0:2222 ./target/release/eipi
```

This is a fun weekend project, not a whistleblower platform. If you need true anonymity, use Tor or don't even use eipi i'm not tryna argue on this. 

</details>

<details>
<summary><strong>Was this vibe coded?</strong></summary>

No. I built this because I wanted to learn how to build SSH apps in Rust using [russh](https://github.com/Russh/russh) and [ratatui](https://github.com/ratatui/ratatui). My friends loved the idea so I ended up publishing it. Did I use LLMs at some points to help? Yeah, the same way I use Google, Stack Overflow, or docs. Every search engine has AI in it these days. I personally [don't like the idea of vibe coding](https://www.pwnwriter.me/syndications/ai-agents). The code is fully open source, go read it.

</details>

## Contributing

Pull requests and contributions are welcome by all means. Feel free to [open an issue](https://github.com/pwnwriter/eipi.boo/issues/new) or submit a PR.

## License

[MIT](LICENSE)

<p align="center"><img src="https://raw.githubusercontent.com/catppuccin/catppuccin/main/assets/footers/gray0_ctp_on_line.svg?sanitize=true" /></p>
<p align="center">Copyright &copy; 2026 - present <a href="https://pwnwriter.me" target="_blank">pwnwriter</a></p>
