# rustcommon-waterfall

Waterfall visualization from heatmap data

## Overview

This library is used to render waterfall visualizations from heatmap data. This
style of visualization represents time moving downwards with values increasing
from left to right. The color represents the density of samples of the same
value within the heatmap.

You can generate some examples with `cargo run --example simulator` which will
produce several example waterfalls from synthetic data.

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

Copyright 2019-2020 Twitter, Inc.

Licensed under the Apache License, Version 2.0:
https://www.apache.org/licenses/LICENSE-2.0

## Security Issues?

Please report sensitive security issues via Twitter's bug-bounty program
(https://hackerone.com/twitter) rather than GitHub.

[contributors]: https://github.com/twitter/rustcommon/graphs/contributors?type=a
[Open Source Code of Conduct]: https://github.com/twitter/code-of-conduct/blob/master/code-of-conduct.md
