> **Warning**  
> this project under development, **Using it is your responsibility**.

<div align="center">
<img src="assets/logo.png" h alt="MetaSSR">
<p align='center'> SSR framework for React.js built on <a href="https://github.com/metacall/core">MetaCall</a> </p>
</div>


MetaSSR is a powerful Server-Side Rendering (SSR) framework crafted for high-performance, dynamic web applications. Built on Rust and leveraging the Axum web framework, MetaSSR integrates seamlessly with the Metacall Runtime, showcasing a real-world use case for polyglot programming. It was created as part of [Google Summer of Code 2024](https://summerofcode.withgoogle.com/archive/2024/projects/yRWw2gPh) by [Mohamed Emad](https://github.com/hulxv), to demonstrate the capabilities of polyglot programming.

## Why MetaSSR?

MetaSSR delivers exceptional performance that sets it apart from traditional SSR solutions. Built with Rust and optimized for speed, it significantly outperforms conventional Node.js-based SSR frameworks.

Here's how MetaSSR compares to Next.js under high load (12 threads, 1000 connections, 30s):

<center>

| Metric              |  MetaSSR  |   Next.js    | Performance Gain |
| ------------------- | :-------: | :----------: | ---------------- |
| **Requests/sec**    | 98,420.11 |   3,170.95   | **31x faster**   |
| **Average Latency** |  8.63ms   |   119.90ms   | **14x lower**    |
| **Transfer/sec**    |  4.98GB   |   37.95MB    | **134x higher**  |
| **Total Requests**  | 2,962,418 |    95,326    | **31x more**     |
| **Max Latency**     |  65.22ms  |    1.99s     | **30x lower**    |
| **Socket Errors**   |     0     | 239 timeouts | **Zero errors**  |

</center>



## Key Features

- **Rust-Powered Performance**: Enjoy the speed and safety of Rust in your server-side rendering tasks.
- **High Performance**: Achieve fast load times and excellent user experiences with optimized server-side rendering.
- **Comprehensive CLI**: Manage your MetaSSR projects effortlessly using our powerful command-line interface.
- **API Route with Polyglot Programming (SOON)**: Integrate multiple languages seamlessly with Metacall's support.

## Getting Started

To get started with MetaSSR, follow these steps:

1. **Installation**: Review our [Installation Guide](docs/getting-started/installation.md) to install MetaSSR on your system.
2. **CLI Documentation**: Learn how to utilize the MetaSSR CLI with our [CLI Documentation](docs/getting-started/cli.md).

## Contributing

We welcome contributions from the community! If you’re interested in helping out, please check out our [Contributing Guide](CONTRIBUTING.md) for information on how to get involved.

## Development

For running a development environment:

### Docker

```sh
docker build -t metacall/metassr:dev -f Dockerfile.dev .
docker run --rm -it metacall/metassr:dev bash
```

### Nix

```sh
TODO
```

## Code of Conduct

To ensure a positive and inclusive environment, please review our [Code of Conduct](CODE_OF_CONDUCT.md).

## Community

- **Discussion Forum**: [Join the Conversation](https://github.com/metacall/metassr/discussions)
- **Twitter**: [Follow US](https://twitter.com/metacallio)
- **Metacall Community**: 
  - [Discord](https://discord.gg/upwP4mwJWa)
  - [Telegram](https://t.me/joinchat/BMSVbBatp0Vi4s5l4VgUgg)
  - [Matrix](https://matrix.to/#/#metacall:matrix.org)

## License

MetaSSR is licensed under the [MIT License](LICENSE). See the LICENSE file for more details.
