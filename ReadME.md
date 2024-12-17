
# Preparing the workspace

## Installation

First you will need to have the Rust toolset installed on your machine. Rust is installed via the rustup installer, which supports installation on Windows, macOS, and Linux.
<https://rustup.rs/>
When you install Rust with rustup, the toolset includes the rustc compiler, the rustfmt source code formatter, and the clippy Rust linter.
You also get Cargo, the Rust package manager, to help download Rust dependencies and build and run Rust programs.
You'll find that you end up using cargo for just about everything when working with Rust.

## Install the rust-analyzer extension

You can find and install the rust-analyzer extension from within VS Code via the Extensions view (Ctrl+Shift+X) and searching for 'rust-analyzer'. You should install the Release Version.
I personally use a few more:

1. Error Lens
2. CodeLLDB
3. Dependi
4. Even Better TOML
5. Rust Syntax
6. Rust
7. vs-code-runner

## Check your installation

After installing Rust, you can check that everything is installed correctly by opening a new terminal/Command Prompt, and typing:

`rustc --version`

which will output the version of the Rust compiler. If you want more details, you can add the --verbose argument. If you run into problems, you can consult the Rust installation guide.

You can keep your Rust installation up to date with the latest version by running:

`rustup update`

There are new stable versions of Rust published every 6 weeks so this is a good habit

# Working with cargo to build,run and run tests

`cargo new` - creates the scaffolding to a new project

`cargo build` - builds the application

`cargo run` - runs the application, to stop it just press `Ctrl+C`

`cargo test` - runs the tests

`cargo add (dependency name)` - will add the new dependecy to your cargo.toml and download it.

`cargo add (dependency name) --features (feature list)` - will ad the dependency with the specified features.

## Deep level testing with log

If you want to have logs on the unit test running with cargo test first install bunyan with `cargo install bunyan`

and then run `TEST_LOG=true cargo test subscribe_returns_a_200_for_valid_form_data | bunyan` with the last part before the pipe '|' being the name of the specific test, or ommit it completely to run all tests.

if "TEST_LOG" is not true no logs will show up when running cargo test
