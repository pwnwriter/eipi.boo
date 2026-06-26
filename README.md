<div align="center">
  <h1><code>$ ssh eipi.boo</code></h1> 
</div>



https://github.com/user-attachments/assets/c5d62780-fa3a-46f6-9192-b60987526d7d



<details>
<summary><strong>Security & Privacy</strong></summary>

**What "anonymous" means here:** Other users can't see who posted what. There are no usernames, no accounts, no profiles. Your confessions, votes, and replies are not tied to any visible identity.

**What the server can see:**
- Your SSH public key fingerprint (SHA-256):  used for rate limiting and vote deduplication, not displayed anywhere
- Your IP address: visible in connection logs like any server, not stored in the database, i myself won't even go ahead and read those ip's unless something happens on the server usually. 
- Your confessions, votes, and replies, stored in SQLite with only the fingerprint as a `hash` not raw fingerprint to identify the auther, btw i public my gpg and ssh public key since 5 years and haven't happend anything wrong yet. 

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
</details>

<details>
<summary><strong>Was AI used while building this?</strong></summary>

Yes, AI was used as an assistant, not as the author. This isn't vibe coded. I know what every function does and why it's there. I built this to learn how SSH apps work in Rust using [russh](https://github.com/Russh/russh) and [ratatui](https://github.com/ratatui/ratatui). My friends loved the idea so I ended up publishing it. I used LLMs the same way I use docs or Stack Overflow, to write better code, not to write code for me. I personally don't like vibe coding either. 
</details>

## Contributing

Pull requests and contributions are welcome by all means. Feel free to [open an issue](https://github.com/pwnwriter/eipi.boo/issues/new) or submit a PR.

## License

[MIT](LICENSE)

<p align="center"><img src="https://raw.githubusercontent.com/catppuccin/catppuccin/main/assets/footers/gray0_ctp_on_line.svg?sanitize=true" /></p>
<p align="center">Copyright &copy; 2026 - present <a href="https://pwnwriter.me" target="_blank">pwnwriter</a></p>
