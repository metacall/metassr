# Improve MetaSSR

# **(MetaCall Server-Side Rendering)**

# While the project has proven its potential in performance benchmarks, it currently faces a number of challenges, including failing tests and stability issues in development mode

# My goal is to make **MetaSSR** reach its full potential

# **About Me**

**Fahd Ashour**, A student studying Commerce at PortSaid University. Always loved computers, self-studied software engineering and did a bunch of web apps. I have interest in going low-level more and more to build the architecture I use when I build high-level applications, I built a lot of web apps, so when I found out about **MetaSSR** I loved it because it’s exactly the thing I wanted to build. I want to participate in **GSoC** with **MetaCall** because it’ll be important for my career and a way to fund my passion in learning and discovering computers and programming. I am enthusiastic about Open Source and recently system programming, I can spend hours and hours reading about the history of computers and how Linus and Stallman argued about licenses, in the same time i can watch [Ryan Carniato talk about web frameworks for 5 hours](https://www.youtube.com/watch?v=oOhfZtUm-pE) and [Jon Gjengset explain Rust and FFI](https://www.youtube.com/watch?v=pePqWoTnSmQ). These are the stuff I'm passionate about, and want to be a part of by going technically deep enough. **Working on MetaCall and MetaSSR gives me the perfect mix of things I’m interested in to work on.** Funding this with GSoC so I can focus on it is a dream.

## Contact Me

* **Email:** [fahd.fady212@gmail.com](mailto:fahd.fady212@gmail.com)  
* **GitHub:** [https://github.com/fahdfady](https://github.com/fahdfady)

# **Project Abstract**

**Project Size:** Large (350h)  
**Idea: [5\. MetaCall SSR (Server-Side Rendering) Server](https://github.com/metacall/gsoc-2026?tab=readme-ov-file#5-metacall-ssr-server-side-rendering-server)**

MetaSSR now does server-side rendering, has a CLI, a dev mode, and dynamic routing. But still, those features need improvements and making it more production-ready. And new features to achieve the real edge of MetaSSR:  **Polyglot Programming**. The project probably will imply some contributions and changes in the MetaCall Rust port.  
I also want to rethink the Pages Router approach and maybe use App Router since we’re adding the API handlers that’ll make more sense.

## **Objectives** (in a checklist)

* [ ] Polyglot API Handler  
      - [ ] Support NodeJS  
      - [ ] Support TypeScript  
      - [ ] Support Python  
      - [ ] Support Ruby  
      - [ ] Support Rust  
* [ ] Improve dev mode  
      - [ ] Real HMR, for now we just rebuild (the whole project) and connect a websocket to refresh the page, this makes file changes trigger a full reload for the page, we’re still far from HMR.  
      - [ ] Specify Server vs Client rebuilds  
      - [ ] Make a dedicated `run dev` command. For now it builds the project and waits for rebuilds.  
      - [ ] Granular rebuilds  
            - [ ] Layout  
            - [ ] Styles  
            - [ ] Page  
            - [ ] Component  
            - [ ] API handler  
* [ ] Middleware for security (continue the work based on the [Middleware RFC by Khaled](https://github.com/metacall/metassr/issues/99))  
* [ ] Add Config file [`config.metassr.js`](http://config.metassr.js)  
      - [ ] Custom port for the server  
      - [ ] Custom port for the devmode websocket connection  
      - [ ] Logging  
      - [ ] Middleware  
      - [ ] dist dir  
      - [ ] Image allowed URLs (like Nextjs)  
      - [ ] Override our opinionated Rspack config  
* [ ] Add testing  
      - [ ] Live reload  
      - [ ] Rebuilding (we already have building unit tests)  
      - [ ] Integration test for running inside docker  
            - [ ] Server starts and responds  
            - [ ] 404 pages work  
            - [ ] Static file received  
            - [ ] Polyglot API handlers work
* [ ] Build an Example of a Polyglot web application, Authentication and everything.

# **Timeline**

The timeline should not be strict, I have exams that might take some time unexpectedly, also We can finish a lot of work unexpectedly in a good way . However, It’s a good way to plan what’s required in x amount of time and to measure how far we are.  
GSoC also provides extensions, I assume that I’ll finish the project by 24 Aug 2026\. But We can still extend it if we get a lot of blockers.

* **Week 1-3**  
  * Community bonding  
* **Week 4-6**  
  * **(I’ll most probably have Final exams here)**  
  * I need to familiarize myself with MetaCall core code (written in C) and some Rust language and FFI because I find that I can be stuck writing the code about polyglot programming.  
  * Work on the Polyglot API handler, support JS, Ruby, Python essentially  
* **Week 6-8**  
  * Make dev mode a separate command and fix its issues  
  * HMR  
    * start by planning the way we’ll handle HMR as mentioned above, most probably we’ll use the way famous frameworks do.  
  * Start implementing Granular rebuilding  
    * Implement for Pages  
    * Layout (full application)  
    * API handler (when an API handler changes update it, don’t wait for a full server restart)  
  * Add unit tests for rebuilding  
* **Week 9**  
  * Document Rebuilder (in its new suit)  
  * Specify Server vs Client rebuilds  
* **Week 10**  
  * **Midterm evaluation**  
  * Add support for Rust  
* **Week 11-14**  
  * Work on the Middleware  
    * Includes re-evaluating the RFC  
  * Add tests  
    * Live reload  
    * Integration test for running inside docker  
      * Server starts and responds  
      * 404 pages work  
      * Static file received  
      * Polyglot API handlers work  
  * Add a configuration file (maybe a `.js` file ? maybe a .toml file ?)  
* **Week 15 \- 16**  
  * Build an Example of a Polyglot web application using MetaSSR.  
  * Final report

# **Why Me**

# I’m passionate about programming, and participating in this project will give me the opportunity to read more and implement low-level, dev-tooling, and web stuff . I’ve talked with Vicente a lot (since last Aug.) and helped me a lot with contributions related to the Rust port and MetaSSR. I'd like to continue having this mentorship experience throughout the GSoC period

I plan to learn by: contributing to MetaSSR & Metacall Rust port, reading a lot (source code from other projects, specifications about frameworks including historical contexts), communicating with my mentor Vicente about any blockers, discussions, and recommendations on material I should consume.I want to write blog posts like what [Francisco did last year with Rust lang](https://web.tecnico.ulisboa.pt/francisco.t.gouveia/posts/01-making-rustup-concurrent/) throughout the GSoC mentorship period but I still don’t know how that fits in the timeline.

# Usage of LLMs

I use LLMs and Agents in my flow. Used [Codex](https://github.com/openai/codex) in the journey of discovering the binutils and Linux codebases. I use Agents while coding too, but I try to limit that to give myself the ability to take my time to write the code myself and learn, I feel agents steal that from me and that’s what I need in this period. I intend to use agents in GSoC but I’d make sure of the output and take ownership of it. I can also discuss parts I don’t fully understand with my mentor Vicente.  
I intend to strictly not use any LLM to communicate, or to explain something (to others) that I don’t understand

## Previous Contributions

I love open-source and believe in its philosophy. Open-source projects helped me learn a lot and was a way to build things I never got the chance to build professionally and step out of my comfort zone. I’ve done some contributions that I’m quite proud of and want to share.

### **MetaCall**

FFI Polyglot programming library written in C

#### Rust Port (5 merged PRs)

* Co-Authored metacall-sys crate with Vicente to link MetaCall C Library with the Rust port [fix and refactor: rust port not finding metacall shared libraries and config files \#569](https://github.com/metacall/core/pull/569)  
* [rust metacall-sys crate: enhance searching with RPATH handling for C Libs discovery across platforms (binary) and improve metacall-sys cargo \#576](https://github.com/metacall/core/pull/576)

#### MetaSSR (10 merged PRs)

Web framework written in Rust that uses Metacall

* [feat: init API system with new crate metassr-api-handler with only GET & POST support for javascript\#67](https://github.com/metacall/metassr/pull/67)  
* [feat: development mode with hot-reloading \#36](https://github.com/metacall/metassr/pull/36)  
* [feat: added interactivity to metassr create subcommand using inquire crate \#45](https://github.com/metacall/metassr/pull/45)

### **Wild**

A fast linker written in Rust

* [feat: Support building on Windows\#1629](https://github.com/wild-linker/wild/pull/1629)  
* I talked with David about LTO, because of [an issue](https://github.com/wild-linker/wild/issues/1646) that complained about not being able to link neovim (my editor of choice, which made me excited to fix that issue and make Wild link it) But didn't fix the issue yet but I'm glad because I got to read about lld LTO implementation and the `SAVE_DIR` utility Wild offers.

### **Cargo**

The Rust package manager

* Fixed a symlink bug [Don't read the config file twice when $CARGO\_HOME is a symlink \#16325](https://github.com/rust-lang/cargo/pull/16325)

### **Turso**

SQLite rewrite in Rust

* [support `format()` function \#4062](https://github.com/tursodatabase/turso/pull/4062)  
* [Rust Bindings: remove unnecessary String allocations using Cow \#4060](https://github.com/tursodatabase/turso/pull/4060)
