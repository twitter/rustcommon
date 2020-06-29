# rustcommon-metrics

The `rustcommon-metrics` crate provides metrics aggregation and reporting

## Overview

This crate defines several metrics sources such as counters, gauges, time
intervals, and other distributions. Channels may be created to hold data from
various sources. Readings are then written into the underlying datastructures
which provide for access to the counters and percentile distributions. 

One of the key features of this library is that for consecutive readings
recorded for counters, it can calculate the secondly rate and store that into
a histogram. Percentiles exported from the histogram will then report statistics
about the rate of change of the counter. By storing samples at high frequency
and reporting periodically from the histogram, we can get a sense of the
smoothness of the rate of change to the counter. For instance, recording
readings every second and reporting minutely provides clues to sub-minutely
characteristics.

Similarly, percentiles of instantaneous gauge readings are also provided. These
can be used to get a sense of how steady the gauge reading is.

This library has a strong focus on performance and is intended to be used to
provide statistics for applications such as benchmarking in rpc-perf as well as
for aggregating samples in Rezolus.

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
