# RHKD

Rust Hotkey Daemon that reads the keyboard event socket


# Configuring

- Set your log file path in the source code, only the output of the commands will be logged
- Set your keyboard fd in the source code (you require root access this file)
- Bind your keys in the bind function (current keycodes are for a french keyboard)
