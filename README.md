refresh-auth-sock
=================

refresh-auth-sock is a small command-line utility written in Rust that helps you discover
SSH agent UNIX domain sockets on the system (typically under /tmp), pick one, and print an
export command so you can update your shell's SSH_AUTH_SOCK to point at that agent.

Why
---
When you run multiple ssh-agent instances (or tools that create agent sockets), your
SSH_AUTH_SOCK may point to an old or dead socket. This tool lists candidate sockets, shows
their modification time, lets you choose one interactively, and outputs the export line
so you can switch your shell to use the selected agent.

Features
--------
- Recursively searches /tmp (skipping unreadable dirs) for socket files whose names start
  with "agent.".
- Shows a small table with an index, whether the socket matches the current SSH_AUTH_SOCK,
  last modification time, and the socket path.
- Lets you choose a socket by number and then prints an export command suitable for
  evaluating in your shell.

Build
-----
You need Rust and Cargo installed. The project uses edition = "2024" in Cargo.toml.

Build a release binary with:

    cargo build --release

The compiled binary will be in target/release/refresh-auth-sock.

Usage
-----
Run the program from your terminal:

    ./target/release/refresh-auth-sock

It will print a table of found sockets and prompt you to enter the socket number or
q to quit. After selecting a socket the program prints a line like:

    export SSH_AUTH_SOCK=/tmp/agent.1234

To actually update your current shell environment, evaluate the program output. For
example, in bash / zsh:

    eval $(./target/release/refresh-auth-sock)

Notes
-----
- The program searches /tmp only. If your system places agent sockets elsewhere you can
  run it from a modified binary (or change the source) to include other paths.
- The program cannot change environment variables in the parent shell directly. Use
  eval as shown above to apply the printed export command to your shell.
- The tool will skip directories it cannot read (permission errors) but will report
  other IO errors to stderr.

Contributing
------------
Patches and suggestions are welcome. Keep changes small and focused.

License
-------
No license is included in this repository.
