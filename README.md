> **Warning**  
> this project under development, **Using it is your responsibility**.

<div align="center">
<img src="assets/logo.svg" alt="MetaSSR">
<p align='center'> SSR framework for React.js built on <a href="https://github.com/metacall/core">MetaCall</a> </p>
</div>

MetaSSR is a powerful experimental Server-Side Rendering (SSR) framework crafted for high-performance, dynamic web applications. MetaSSR uses Metacall Runtime, exploring web-based use cases for polyglot programming.

## Why MetaSSR?

MetaSSR has a great performance potential, performing significantly better than other more mature production frameworks.

Here's how MetaSSR compares to Next.js under high load (12 threads, 1000 connections, 30s). The numbers below come from running both frameworks' published npm packages in Docker containers on the same machine — MetaSSR `1.0.0-alpha.2` (SSR build) vs Next.js `16.3.6` (production build). Raw output is in [`benchmarks/reports/metassr-vs-nextjs-2026-09-28/`](benchmarks/reports/metassr-vs-nextjs-2026-09-28/).

### API route (`/api/bench` — identical API handler and payload in both apps)

<center>

| Metric              |  MetaSSR  |   Next.js   | Performance Gain |
| ------------------- | :-------: | :---------: | ---------------- |
| **Requests/sec**    | 100,103.87 |   1,376.68  | **73x faster**   |
| **Average Latency** |  11.59ms  |  225.13ms   | **19x lower**    |
| **Transfer/sec**    |  15.85MB  |  428.87KB   | **37x higher**   |
| **Total Requests**  | 3,006,409 |   41,410    | **73x more**     |
| **Max Latency**     |   1.05s   |    1.90s    | 1.8x lower       |
| **Socket Errors**   |     0     | 137 (12 read + 125 timeouts) | **Zero errors** |

</center>

### SSR page (`/` — both apps render the same 20-item list on every request)

<center>

| Metric              |  MetaSSR  |   Next.js   | Performance Gain |
| ------------------- | :-------: | :---------: | ---------------- |
| **Requests/sec**    | 206,247.56 |    422.81   | **488x faster**  |
| **Average Latency** |   7.20ms  |  745.08ms   | **103x lower**   |
| **Transfer/sec**    | 597.16MB  |   4.83MB    | **124x higher**  |
| **Total Requests**  | 6,207,426 |   12,720    | **488x more**    |
| **Max Latency**     |   1.03s   |    1.99s    | 1.9x lower       |
| **Socket Errors**   |     0     | 152 timeouts | **Zero errors**  |

</center>

## Key Features

- **Rust-Powered Performance**: Enjoy the speed and safety of Rust in your server-side rendering tasks.
- **High Performance**: Achieve fast load times and excellent user experiences with optimized server-side rendering.
- **API Route with Polyglot Programming**: Integrate multiple languages in the backend with Metacall's support.

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
- **Metacall Community**:
  - [Discord](https://discord.gg/upwP4mwJWa)
  - [Telegram](https://t.me/joinchat/BMSVbBatp0Vi4s5l4VgUgg)
  - [Matrix](https://matrix.to/#/#metacall:matrix.org)

## License

MetaSSR is licensed under the [MIT License](LICENSE). See the LICENSE file for more details.
