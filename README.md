# Intro-To-Software-Projects

## Building Rust Code

The Rust code in the `rust/` folder is responsible for rendering the terminal side of this game. You can compile it by installing Rust [here](https://rustup.rs) or with [winget](https://winstall.app/apps/Rustlang.Rustup) if you're on Windows.

Once you've installed Rust, you can compile the code with
```shell
cd rust/
cargo build
```

And you're off to the races. The Godot project can just be opened in the Godot editor once the Rust code is compiled.

## Documenting Rust Code

The Rust code provides Godot classes that can be brought into the scene tree and used in the Godot project. To document the classes and their associated fields and functions, run these commands

```shell
cd rust/
cargo doc --open
```

This will open the documentation for the Rust code in a web browser.

