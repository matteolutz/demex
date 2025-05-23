![demex](./assets/LogoV1-Wide-Title.png)
[![Codacy Badge](https://app.codacy.com/project/badge/Grade/a5327de0b1c14145ab6c275f1d3b1431)](https://app.codacy.com/gh/matteolutz/demex/dashboard?utm_source=gh&utm_medium=referral&utm_content=&utm_campaign=Badge_grade)

## What is demex?

demex is a command based DMX-Lighting control application written in Rust. It's command syntax and functionality is heavily inspired by the _ETC Eos_ family and _MA_ family of lighting consoles.

## Getting started

demex is currently in development and not yet ready for use. If you want to try it out, you can clone the repository

```bash
git clone https://github.com/matteolutz/demex
```

and run the project using

```bash
cargo run -- -s test_data/cinema.json
```

There will be test data loaded and you can start playing around with the commands and the UI.

### Installation

#### Ubuntu / Debian

Before you can run demex, you need to install some additional packages. You can do this by running

```bash
sudo apt-get install g++ pkg-config libx11-dev libasound2-dev libudev-dev libxkbcommon-x11-0
```

#### Windows
> TODO

## Contributing

Pull requests are welcome. For major changes, please open an issue first to discuss what you would like to change.

## License

This project is licensed under the MIT License. See [LICENSE](LICENSE) for more informations.

---

> [matteolutz.de](https://matteolutz.de) &nbsp;&middot;&nbsp;
> GitHub [@matteolutz](https://github.com/matteolutz) &nbsp;&middot;&nbsp;
> Email [info@matteolutz.de](mailto:info@matteolutz.de)
