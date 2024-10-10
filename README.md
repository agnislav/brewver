# Brewver

Brewver is a command-line tool written in Rust that allows you to install a specific version of a Homebrew formula. It fetches the desired version from the Homebrew repository and installs it on your system.

## Features

- Install specific versions of Homebrew formulas.
- Automatically fetches the correct commit and bottle file from the Homebrew repository.
- Provides detailed logging for debugging and information purposes.

## Prerequisites

- Rust and Cargo installed on your system. You can install them from [rustup.rs](https://rustup.rs/).
- Homebrew installed on your system. You can install it from [brew.sh](https://brew.sh/).

## Installation

1. Clone the repository:

   ```bash
   git clone https://github.com/agnislav/brewver.git
   cd brewver
   ```

2. Build the project using Cargo:

   ```bash
   cargo build --release
   ```

3. The compiled binary will be located in the `target/release` directory. You can move it to a directory in your `PATH` for easier access:

   ```bash
   mv target/release/brewver /usr/local/bin/
   ```

## Usage

To use Brewver, run the following command:

```bash
brewver <formula_name> <formula_version>
```

For example, to install version 1.0 of `openssl`, you would run:

```bash
brewver openssl 1.0
```

## Logging

Brewver uses the `log` crate for logging. The logging level can be set using the `RUST_LOG` environment variable. For example:

```bash
RUST_LOG=debug brewver openssl 1.0
```

Default log level is `info`.


## Contributing

Contributions are welcome! Please fork the repository and submit a pull request with your changes.

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.

## Author

Agnislav Onufriichuk

