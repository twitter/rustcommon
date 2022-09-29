# histogram

A simple histogram implementation inspired by HDRHistogram and our base-2
implementation from [ccommon]. It uses atomic counters to record increments for
each value.

## Getting Started

### Building

rustcommon is built with the standard Rust toolchain which can be installed and
managed via [rustup](https://rustup.rs) or by following the directions on the
Rust [website](https://www.rust-lang.org/).

#### View library documentation
```bash
cargo doc --open
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

Copyright 2020 Twitter, Inc.

Licensed under the Apache License, Version 2.0:
https://www.apache.org/licenses/LICENSE-2.0

## Security Issues?

Please report sensitive security issues via Twitter's bug-bounty program
(https://hackerone.com/twitter) rather than GitHub.

[ccommon]: https://github.com/twitter/ccommon/
[contributors]: https://github.com/twitter/rustcommon/graphs/contributors?type=a
[Open Source Code of Conduct]: https://github.com/twitter/code-of-conduct/blob/master/code-of-conduct.md
