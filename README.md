# Heritage Pathfind

`heritage-pathfind` is a command line utility to parse relationship files
and find ancestry paths.


### Design Goals:

* Fast run time
* Small memory footprint
* Minimal interface

Uses https://github.com/bluss/petgraph to do graph operations like path finding.

## Example

```sh
$ ./heritage-pathfind -r relationship-utf8.csv -c 1 -a 20
-> Name A(20) is Father of
-> Name B(6) is Father of
-> Name C(1)
```

## Getting Started

```
git clone https://github.com/Voultapher/heritage-pathfind.git
cd heritage-pathfind
cargo build --release
./target/release/heritage-pathfind -r relationship-utf8.csv -c 1 -a 20
```

### Prerequisites

Rust toolchain and cargo.

### Installing

[See cargo docs](https://doc.rust-lang.org/cargo/guide/).

## Running the tests

```
cargo test
```

## Contributing

Please read [CONTRIBUTING.md](CONTRIBUTING.md)
for details on our code of conduct, and the process for submitting pull requests to us.

## Versioning

We use [SemVer](http://semver.org/) for versioning. For the versions available,
see the [tags on this repository](https://github.com/Voultapher/heritage-pathfind/tags).

## Authors

* **Lukas Bergdoll** - *Initial work* - [Voultapher](https://github.com/Voultapher)

See also the list of [contributors](https://github.com/Voultapher/heritage-pathfind/contributors)
who participated in this project.

## License

This project is licensed under the Apache License, Version 2.0 -
see the [LICENSE.md](LICENSE.md) file for details.
