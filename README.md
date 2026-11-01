> **Warning**  
> this project under development, **Using it is your responsibility**.

<div align="center">
<img src="assets/logo.svg" alt="MetaSSR">
<p align='center'> SSR framework for React.js built on <a href="https://github.com/metacall/core">MetaCall</a> </p>
</div>

MetaSSR is a powerful experimental Server-Side Rendering (SSR) framework crafted for high-performance, dynamic web applications. MetaSSR uses MetaCall Runtime, exploring web-based use cases for polyglot programming.

## Why MetaSSR?

MetaSSR has a great performance potential, performing significantly better than other more mature production frameworks.

Here's how MetaSSR compares to Next.js under high load (12 threads, 1000 connections, 30s). Both frameworks run in Docker with host networking on the same machine. MetaSSR is built from source at commit `6879584` (includes the per-request SSR fix #220); Next.js is pinned to `16.3.7`. All CPUs are capped at 2.5 GHz for both sides to avoid thermal throttling. The report is in [`benchmarks/reports/metassr-vs-nextjs-2026-10-31/`](benchmarks/reports/metassr-vs-nextjs-2026-10-31/). Raw output is generated locally and is not committed.

### API route (`/api/bench` — equivalent JSON handler and payload in both apps)

<center>

| Metric              |   MetaSSR   |   Next.js   | Performance Gain |
| ------------------- | :---------: | :---------: | ---------------- |
| **Requests/sec**    | 107,116.71  |   1,469.79  | **73x faster**   |
| **Average Latency** |   10.15ms   |  126.75ms   | **12x lower**    |
| **Transfer/sec**    |   16.96MB   |  457.89KB   | **38x higher**   |
| **Total Requests**  |  3,216,110  |   44,239    | **73x more**     |
| **Max Latency**     |   71.29ms   |    1.97s    | 28x lower        |
| **Timeouts (2s)**   |      0      |     237      | **Zero timeouts** |

</center>

### SSR page (`/` — both apps render the same 20-item list on every request)

<center>

| Metric              |  MetaSSR   |  Next.js  | Performance Gain |
| ------------------- | :--------: | :-------: | ---------------- |
| **Requests/sec**    | 41,499.66  |  500.86   | **83x faster**   |
| **Average Latency** |   25.36ms  | 495.70ms  | **20x lower**    |
| **Transfer/sec**    |  120.16MB  |   5.72MB  | **21x higher**   |
| **Total Requests**  | 1,249,104  |  15,057   | **83x more**     |
| **Max Latency**     |  158.38ms  |   1.86s   | 12x lower        |
| **Timeouts (2s)**   |     0      |     234      | **Zero timeouts** |

</center>

> **Note on older numbers:** the previous README figures came from the `2026-09-28` run, which went through Docker's userland proxy and is invalid. The corrected `2026-10-03` run used npm `1.0.0-alpha.2`, which predates the per-request SSR fix, so its SSR row measured boot-time cached HTML. The `2026-10-31` report above is the first run that measures real per-request SSR on current source.

## Key Features

- **Rust-Powered Performance**: Enjoy the speed and safety of Rust in your server-side rendering tasks.
- **High Performance**: Achieve fast load times and excellent user experiences with optimized server-side rendering.
- **API Route with Polyglot Programming**: Integrate multiple languages in the backend with MetaCall's support.

## Getting Started

To get started with MetaSSR, follow these steps:

1. **Installation**: Review our [Installation Guide](docs/getting-started/installation.md) to install MetaSSR on your system.
2. **CLI Documentation**: Learn how to utilize the MetaSSR CLI with our [CLI Documentation](docs/getting-started/cli.md).

## Contributing

We welcome contributions from the community! If you're interested in helping out, please check out our [Contributing Guide](CONTRIBUTING.md) for information on how to get involved.

For running a development environment:

### Docker

```sh
docker build --target build_debug -t metacall/metassr:dev -f Dockerfile.dev .
docker run --rm -it metacall/metassr:dev bash
```

### Nix

1. **Install Nix** (if not already installed):

   ```bash
   sh <(curl --proto '=https' --tlsv1.2 -L https://nixos.org/nix/install) --daemon
   ```

2. **Enable Nix Flakes**:

   ```bash
   mkdir -p ~/.config/nix
   echo "experimental-features = nix-command flakes" >> ~/.config/nix/nix.conf
   ```

3. **Enter Development Shell**:

   ```bash
   nix develop
   ```

This will automatically set up Rust, MetaCall, and all required dependencies, same dependencies for all of us developing.

## Code of Conduct

To ensure a positive and inclusive environment, please review our [Code of Conduct](CODE_OF_CONDUCT.md).

## Community

MetaSSR is essentially a community project, initially created as part of [Google Summer of Code 2024](https://summerofcode.withgoogle.com/archive/2024/projects/yRWw2gPh) by [Mohamed Emad](https://github.com/hulxv), to demonstrate the capabilities of polyglot programming.

- **Discussion Forum**: [Join the Conversation](https://github.com/metacall/metassr/discussions)
- **Twitter**: [Follow US](https://twitter.com/metacallio)
- **MetaCall Community**:
  - [Discord](https://discord.gg/upwP4mwJWa)
  - [Telegram](https://t.me/joinchat/BMSVbBatp0Vi4s5l4VgUgg)
  - [Matrix](https://matrix.to/#/#metacall:matrix.org)

## License

MetaSSR is licensed under the [MIT License](LICENSE). See the LICENSE file for more details.
