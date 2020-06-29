# rustcommon

rustcommon is a collection of common libraries we use in our Rust projects. This
includes datastructures, logging, metrics, timers, and ratelimiting.

## Overview

rustcommon is a workspace repository which contains several crates (libraries)
which we use in our Rust projects such as rpc-perf and Rezolus. These common
libraries may be useful in other projects, and as such, we are providing them
here for ease of discovery.

Each crate within this repository contains its own readme and changelog
detailing the purpose and history of the library.

## Getting Started

### Building

rustcommon is built with the standard Rust toolchain which can be installed and
managed via [rustup](https://rustup.rs) or by following the directions on the
Rust [website](https://www.rust-lang.org/).

#### Clone and build rustcommon from source
```bash
git clone https://github.com/twitter/rustcommon
cd rustcommon

# run tests
cargo test --all
```

## Support

Create a [new issue](https://github.com/twitter/rustcommon/issues/new) on GitHub.

## Contributing

We feel that a welcoming community is important and we ask that you follow
Twitter's [Open Source Code of Conduct] in all interactions with the community.

## Authors

* Brian Martin <bmartin@twitter.com>

A full list of [contributors] can be found on GitHub.

Follow [@TwitterOSS](https://twitter.com/twitteross) on Twitter for updates.

## License

Copyright 2019-2020 Twitter, Inc.

Licensed under the Apache License, Version 2.0:
https://www.apache.org/licenses/LICENSE-2.0

## Security Issues?

Please report sensitive security issues via Twitter's bug-bounty program
(https://hackerone.com/twitter) rather than GitHub.

[contributors]: https://github.com/twitter/rustcommon/graphs/contributors?type=a
[Open Source Code of Conduct]: https://github.com/twitter/code-of-conduct/blob/master/code-of-conduct.md
