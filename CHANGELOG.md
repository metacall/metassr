## [1.0.0-alpha.2] - 2026-09-26

### 🚀 Features

- *(cli)* Add --install option to metassr create (#202)
- *(agent)* Example-gallery workflow — skill, AGENTS.md, loader metadata (#204)
- Ship the Python loader in the npm payload and release (#206)

### 🔨 Refactor

- *(docker)* Retire base image, npm-install metassr in app image (#203)

### 🔁 CI

- *(npm)* Support manual dispatch with a version input

### 💼 Other

- Document ~/.metassr directory
- Update npm package README
- Cleanup install-metacall.sh and rspack logo

## [1.0.0-alpha.1] - 2026-09-25

### 🚀 Features

- *(docker)* Add MetaSSR base image (#194)
- *(docker)* Add generic app image (#195)
- *(docker)* Compile dependencies with cargo-chef (#196)
- *(site)* Add landing site deployed to GitHub Pages (#197)
- *(site)* Redesign landing page with MetaSSR design system (#198)

### 🪲 Bug Fixes

- *(npm)* Assemble-payload remove hardlinks due to npm annyoed about hardlinking esbuild

### 🧹 Chores

- Inherit workspace version in metassr-api-handler (#190)
- *(scripts)* Add pinned MetaCall installer (#191)
- *(docker)* Bump MetaCall runtime to 0.9.24 (#199)

### 💼 Other

- Add non-background logo (#188)
- Update Docs (#189)

* edit README.md

* edit TODO.md
- Publish MetaSSR on npm (#200)

* chore(docker): bump MetaCall runtime to 0.9.24

* publish metassr on npm.

npm package: metassr -> CLi
npm package: metassr linux-64 with glibc package for prebuild deps
- Update metassr-create templates' json scripts to match the current state of the project (#201)

## [pre-release] - 2026-08-23

### 🚀 Features

- Implement logging layer for [tracing](https://github.com/tokio-rs/tracing)
- *(cli)* Integerate with logger layer and impl http tracing
- Serving `/static` directory
- Adding  filesystem anaylzer
- Building the html-generator crate
- Feat(builder)t: make the client bundler takes multiple entries at once
- *(builder)* Implement the page hydrator (works at build time)
- *(utils)* Custom implementation for caching directory
- Implement the client builder
- Implementing server side rendering (#2)
- *(html-gen)* Impl `ToString` for HtmlOutput`
- *(builder)* Impl targets generator
- *(builder)* Impl manifest file
- *(builder)* Impl page renderer
- *(server)* Rendering page on request
- Feat(utils: make CacheDir::new() accpet generic type
- *(builder)* Impl static-site generation
- *(server/router)* Impl `fallback` for `RouterMut`
- *(server)* Serving generated static-site
- *(cli)* Add an option for building and serving generated static-site app
- *(metassr-create)* Include templates at compilation-time
- *(metassr-create)* Add javascript template
- *(metassr-create)* Add typescript template
- *(creator)* Implement loading templates
- *(creator)* Impl web application creator
- *(cli)* Create command
- *(builder)* Render `head` at build-time
- *(cli)* Panic when no subcommand is provided
- *(bundler)* Catching errors from `rspack` compiler
- Feat(utils): implement some traits for Rand struct
- implement `std::fmt::Display`
- implement `std::cmp::PartialEq` and `std::cmp::PartialOrd`
- implement `std::cmp::PartialEq<i64>` and `std::cmp::PartialOrd<i64>`
- Feat(cli): improve building-time logging
inform the user of the time of each build operation
- Add interactivity to `create` subcommand using inquire crate (#45)
- Development mode with hot-reloading (#36)
-  feat: add coloring and style to create interactive command (#56)

* Add feature: add a specific color for each template

* make fmt containing the whole logic of coloring templates

---------

Co-authored-by: Mohamed Emad <hulxxv@gmail.com>
- Init API system with new crate `metassr-api-handler` and a small hello.js test
- Setup benchmarking workflow (#63)
- Try incrementing server port if 8080 (or the chosen port via cli) instead of failing (#164)
- Support PUT and DELETE HTTP Methods (#167)
- Compile error overlay (#78)
- Add standalone performance benchmark suite (#171)
- Add baseline comparison to benchmark CI (#178)
- Feat (metassr-config): add config loading logic and testings (#180)

* feat (metassr-config): add config loading logic and unit tests

* test(web-app) add metassr.toml and fix wrongly arranged args in `npm run dev` command{

* fix fmt

* feat(metassr-config): load metassr.toml and merge CLI args with config values

* test(config): default the server port to 8080 so it doesn't break integration tests on the CI
- *(cli)* Version argument with cool ASCI art (#185)
- *(bench-ci)* Persist benchmark results and compare against latest master (#186)

### 🪲 Bug Fixes

- Bundling_client() FunctionNotFound
- *(server)* Http-log layer doesn't work
- Fix(clippy)
- *(cli)* Clippy
- *(builder)* `head` is already loaded via metacall
- *(creator)* Cannot include non-utf8 files
- *(docs)* Correct some information
- *(docs)* Correct repo links after rename it
- *(cli)* Return default values for cli
- *(builder)* `render_head` not found
- *(creator)* Exclude `node_modules` and `dist` during loading templates
- Await compilation of bundling function
- Fix(clippy)
- *(utils)* Wrong pathname is registered in the cache
- *(createor)* Invalid comma in `package.json` in ts tempalte
- *(bundler)* Return an error when targets not found
- *(flake)* Edit lib paths
- Fix(build): error[E0618]: expected function, found `metacall::init::MetaCallDestroy`
(Fixes #52)
- Formatting
- Clippy around the codebase (#59)
- Update package version from `0.0.1-alpha` to `1.0.0-alpha`
- Update package versioning to use workspace configuration (#61)
- Incorrect version of `metassr-util`
- Fix warnings
- Hardcoded WebSocket port 3001 (#71)
- *(metassr-html)* Prevent panic in HtmlPropsBuilder when scripts or styles are not set (#83)
- *(metassr-server)* Warn on unimplemented rebuild types in dev mode (#87)
- *(live-reload)* Fix typo causing stylesheet cache-busting to silently fail in dev mode (#105)
- Correct BuildingType mapping for SSR and SSG in builder.rs (#127)
- Live-reload API handlers in src/api (#128)
- *(fs-analyzer)* Avoid panic on extension-less files in src/ (#134)
- Fix js template: running render functions `React is not defined` (#163)

* fix js template: do not import types in jsx files

* fix js template: React not defined

* tests: add Random Users App to tests README.md
- Fix imports
- *(bench-ci)* Ensure benchmark artifacts are uploaded (#176)
- Fix clippy error

### 🔨 Refactor

- Refactoring server runner and make it more clean
- *(utils)* Replace `fs_analyzer` with `src_analyzer` to be more accurate
- Use `PathBuf` instead of strings for paths in the `hydrator`
- Remove the over-engineering in `output.filename`
- Move renderers to a stand-alone directory
- *(builder)* Remove bundling type
- *(builder)* Swap `MultiRenderExec` hashmap keys with values
- *(builder)* Handliing path.strip_prefix error
- *(cli)* Make cli more clean
- *(builder)* Make head rendering more clean
- Move web-bundler to a stand-alone crate called `metassr-bundler`
- *(utils)* Improve docs of `CheckerState` and add some test cases
- *(utils)* Improve `rand::Rand`
- *(utils)* Improve cache directory
- *(utils)* Improve directories analyzer
- Move `metassr_utils::analyzer` to a standalone crate
- *(utils)* Return `&Path` instead of `PathBuf` clone from `CacheDir.dir_path()` and rename it to `CacheDir.path()`
- Rename the bin from `metassr-cli` to `metassr`
- Streamline MetaCall installation in dev shell
- Rename `html-generator` to `metassr-html` (#58)
- Refactor `metassr dev`: make it build nativelty without relying on `metassr build` (#161)

* feat(dev-mode): allow taking optional arg build_type from metassr cli

* add Builder to the dev exec

* change package.json for tests
- Unify BuildingType enum into metassr-build (#187)

### 📚 Documentation

- *(typo)* Correct capitalization error
- *(cli)* Improve `--help`
- Add code of conduct file
- Add a guide for contributing
- MIT license
- Improve README.md
- Getting-started documentation
- Readme file for docs
- Complete `create` command task
- Move license file from markdown to plain-text
- Explain the folder structure of web application
- Add ref for `file-strcuture.md` in `docs/README.md`
- Add some ideas for  features we can implement in the future
- *(utils)* Improve documentation of `metassr-utils` crate
- *(utils)* Improve formatting
- Docs(bundler) : improve docs of web bundler
- *(bundler)* Add some docs to `bundler.js`
- *(bundler)* Remove useless examples
- *(bundler)* Improve docs of `bundle.js`
- Improve readme with adding benchmarks
- Edit README.md & CONTRIBUTING.md with setup instructions (#51)
- *(metassr-cli)* Add missing doc comment for subcommand (#84)
- *(tests)* Add Random Users App to tests README.md

### 🧪 Testing

- Adding main test sample
- Some changes in test sample (static serving)
- Improve test sample
- Improve test sample
- *(metassr-html)* Add unit tests for HtmlBuilder edge cases (#91)
- *(client)* Implement comprehensive test suite for ClientBuilder (#139)
- *(metassr-build)* Cover `setup_page_path` and `Targets` (#156)

### 🔁 CI

- *(rust)* Add GitHub Action for Clippy linting
- Create label.yml
- Improving testing workflow (#46)
- *(bench)* Remove comment

### 🏗️ Builds

- *(nix)* Use `fenix` to manage rust toolchain dependencies (#47)
- Bump metacall to 0.5.6 and solve conflicts (#66)
- *(deps)* Metacall 0.5.10 (#158)

### 🧹 Chores

- Implement basic for development
- Rm garbages
- Remove useless log file
- Initilize metassr-build
- Remove completed todos
- Remove some logging
- Init metassr-create
- Remove `yarn.lock` from templates
- Change .gitignore
- *(creator)* Remove `build.js`
- *(creator)* Remove unused deps in templates
- Bump `metacall` crate  to 0.4.1
- *(build)* Remove `metassr-build/src/bunlder.rs` (it was moved to `metassr-bundler`)
- *(flake)* Add less
- Typos in error messages across multiple files (#125)
- Add __pycache__ to gitignore due to running benchmarks via python
- Update all tsconfig and jsconfig from es2016 to es2018 (#184)

### 💼 Other

- Initilize
- Merge pull request #2 from Hulxv/logger

feat: implementing logger
- Initlize internal compiler with ts-to-js converter
- Adding todo list for main features that will be implemented
- Adding custom fallback page to todo list
- Merge pull request #1 from metacall/metassr-build

feat: metassr internal builder
- Complete SSR feature (#3)

* feat(html-gen): able to save the html output to a file

* refactor(html-gen): avoid  borrow checker headache

* refactor(builder): make consts and traits that shared between client and server builders

* fix(utils): cacher doesn't rewrite the file if a diff detected

* refactor(builder): move scripts outside the client builder

* feat(utils): a simple random values generator

* refactor(utils): move `AnalyzeDir` trait to a stand-alone file contains traits

* feat(utils): make a `dist` directory analyzer

* refactor(builder): use shared traits in the client builder

* feat(builder): implement server side rendering

* feat(cli): add subcommands for building and running

* refactor(build): make web bundler shared between client and server builders

* fix(builder): metacall panics because `web_bundling()` is loaded twice

* refactor(builder): bundling server renderer before rendering html pages

* refactor: drop metassr-swc (useless crate)

* feat(build): read head content from _head.tsx

* test: improve test sample

* chore: cleanup with clippy

* chore(git): add some ignored entries

* feat: implement custom fallback page

* feat: serving server-side rendered pages

* refactor: improve importing special files with src_analyzer

* refactor(builder): improve html rendering process

* refactor(builder): improve getting targets for bundling and render_exec

* refactor(utils): use pathbuf instead of path with lifetime

* fix(builder): use `path.join` to work in cross-platform

* refactor(builder): make pages generating more clean

* refactor: move `metassr-core` to `metassr-server`

* refactor(cli): remove duplicated code

* docs: move TODOs to a stand-alone markdown file
- Merge pull request #4 from metacall/rendering-at-req

feat: add options for SSR and SSG
- Merge pull request #5 from metacall/metassr-create

feat: metassr-create
- Merge pull request #6 from metacall/metassr-create

complete: metassr-create
- Merge pull request #7 from metacall/docs

docs: initial documentation for MetaSSR
- Merge pull request #8 from metacall/fix-head

fix(builder): render_head not found
- Merge pull request #9 from metacall/fix-create

fix(creator): exclude `node_modules` and `dist` during loading templates
- Shell script to update version of crates
- Improving a bit the compilation.
- Trying to improve this again.
- Fea(utils)t: add a simple checker state
- Add a todo to return the error of bundling
- Merge pull request #10 from metacall/fix/async-await-compilation

Fix/async await compilation
- Merge pull request #11 from metacall/improve-utils

refactor: improve `metassr-utils`

- Document all utilities in `metassr-utils`
- Improve tests for all utilites
- move dirs analyzers to `metassr-fs-analyzer`
- Merge pull request #22 from metacall/feat/calc-building-time

feat(cli): improve building-time logging
- Merge pull request #28 from metacall/docs/metassr-bundler

docs: document metassr-bundler
- Merge pull request #30 from metacall/fix/bundler-targets-not-found

fix(bundler): return an error when targets not found
- Add benchmarks, we will migrate this to another repo in a near future.
- Bump to metacall-rs 0.5 (#39)
- Init a working rust nix flake
- Update flake

Signed-off-by: fahdfady <fahd.fady212@gmail.com>
- Try to define metacall in flake

Signed-off-by: fahdfady <fahd.fady212@gmail.com>
- Enhance flake.nix with important pkgs and lib paths
- Merge pull request #40 from fahdfady/nix-flake-support

Nix flake support
- Added docker, ci, and improved code a bit.
- Add docker ci.
- Solve issues.
- Improve docker ci.
- Remove debug in docker image.
- Improve ci.
- Debug workaround, it does not support asan.
- Solve matrix.
- Set up debug again, remove clippy.
- Solve issue with mac.
- Solve clippy issues.
- Solve more issues.
- Revert version rspack.
- Merge pull request #44 from fahdfady/flake-add-less

chore(flake): add less
- Change debug to release in ci.
- Solve memory issue with async api.
- Solve CI and some more issues.
- Solved issues in ci.
- Minor change.
- Make bundle.sh executable.
- Change other issues.
- Improve unit tests.
- Trying to improve ci.
- Remove label.yml.
- Solve issue.
- Add vscode for debug and imporoved ci.
- Improve ci.
- Solve more issues in the ci.
- Solve more issues in ci.
- Remove unnecessary async (#62)
- Add todos
- Disable debugging message
- Refactor handler closure to ignore unused parameters
- Merge
- Fix Ci Tests (#68)

* fix: `metassr-build` tests

* fix: rustfmt

* fix typos

Signed-off-by: Fahd Ashour <fahd.fady212@gmail.com>

* fix metassr_bundler tests by adding NODEPATH env var in dockerfile and removing unncessary metacall init

* fix watcher debouncer cache type across platforms

* fix windows template path escaping in build script

* fix windows integration workflow shell

* fix windows npm script shell in integration workflow

* centralize CLI invocation behind npm metassr scripts. this fixes the windows CI but introduces new things to package.json in testing

* CI: ensure metacall runtime path for integration

* run the samme install command for linux and mac. remove env commands for windows and mac

---------

Signed-off-by: Fahd Ashour <fahd.fady212@gmail.com>
Co-authored-by: Fahd Ashour <fahd.fady212@gmail.com>
- Update integration.yml
- Move benchmarks to https://github.com/metacall/metassr-benchmarks (#76)
- Revert "docs(metassr-cli): add missing doc comment for subcommand (#84)" (#98)

This reverts commit f054b7e55902e3eadc791b62946d172b04dd3bb4.
- Added Dockerfile and Documentation (#85)

* added Dockerfile

* Added expose in Dockerfile

* updated code

* updated code

* added docs to installation.md

* removed deployment section from README.md

* Update Dockerfile

---------

Co-authored-by: Fahd Ashour <fahd.fady212@gmail.com>
Co-authored-by: Vicente Eduardo Ferrer Garcia <7854099+viferga@users.noreply.github.com>
- Revert "fix(live-reload): fix typo causing stylesheet cache-busting to silent…" (#109)

This reverts commit b9da69883c914d0277e3c9c4b6c453068de7e3a8.
- Revert "Revert "fix(live-reload): fix typo causing stylesheet cache-busting t…" (#110)

This reverts commit a350c48b9a28e9d303c5e02c07815fa872288290.
- Replace std::path::Path::canonicalize with dunce for path handling (#115)

* feat:use dunce for path canonicalization

This commit updates several dependencies and introduces the `dunce` crate for canonicalizing file
paths.

Previously, `std::path::Path::canonicalize` was used, which can panic on Windows if the path
contains invalid UTF-8 characters or returns unhandled like `///?`. `dunce` provides a more robust
solution.

The `dunce` crate has been added as a dependency to multiple crates, and its usage has replaced
calls to `canonicalize` where appropriate. This includes:

- `metassr-build`: In `hydrator.rs`, `client/mod.rs`, `server/manifest.rs`, `server/render.rs`,
  `server/renderer/head.rs`, and `server/targets.rs`.
- `metassr-create`: In `build.rs`.
- `metassr-fs-analyzer`: In `dist_dir.rs`.
- `metassr-utils`: In `cache_dir.rs`.

* docs: Clarify Windows path handling "dunce "in CONTRIBUTING.md

* feat(paths): Handle Windows paths for JS

Introduce `metassr_utils::js_path::to_js_path` to correctly format
Windows paths for JavaScript consumption. This involves replacing
backslashes with forward slashes, preventing unintended escape
sequence interpretation.

Also, update documentation and code to use `dunce::canonicalize`
for path canonicalization on Windows.
- Rspack rust implementation (#111)

* pining versions to rspack git version

* compiler init and entry points specified

* compiler run with threads handling

* dist_path cononical, rspack filesystem setup

* jsx, tsx, asset inline support

* moduleoptions added to compiler

* resove, optimization and loader_swc support

* tests

* removed bundle.js

* cargo lock

* uncommented metacall runtime in tests

* added metacall to bundler

* downgraded to stable version of rspack, removed debug statements

* lint

* bumped rspack to the latest version fixing the dependency issue

* added nightly toolchain

* removed the part where we are explicity installing stable rust

* update dockerfile for nightly rust too

* macos build failing due to linker issue

* reverting back linker path macos
- Revert "Rspack rust implementation (#111)" (#117)

This reverts commit 64189a58bf61800512817deb8104f36fd37ad86e.
- Fix macOS CI in integration workflow (#118)

* ci: Refine integration tests for macOS compatibility

Adjust CI workflow to handle macOS-specific build and test configurations.

This change updates the integration test workflow to correctly manage dependencies and execute tests
on macOS runners. It also refactors the installation commands to use `runner.os` for better platform
detection.

Additionally, the `rust-toolchain.toml` file is updated to use the latest nightly channel, ensuring
compatibility with newer dependencies like rspack.

* fix: close rspack compiler after bundling to prevent process hang

compiler.run() leaves internal handles (file watchers, threads)

Follows rspack's documented API contract:
https://rspack.dev/api/javascript-api/compiler#close
- Fixed the failing MacOS runner in CI  (#137)

* updated metacall-sys version to 0.1.5

* testing

* removed the test code

* added metacall installation in lint.yaml

* updated version of metacall from 0.5.6 to 0.5.8

* fix: regenerate Cargo.lock against published crates

* added libclang-dev in Dockerfile.dev

* Update lint.yml

---------

Co-authored-by: Vicente Eduardo Ferrer Garcia <7854099+viferga@users.noreply.github.com>
- Add unit tests for metassr-server and metassr-api-handler (#130)

* fix: correct BuildingType mapping for SSR and SSG in builder.rs

This commit fixes the mapping of string representations to the BuildingType enum. The previous implementation incorrectly mapped "ssr" to Ssg and "ssg" to Ssr. The corrected mappings now properly associate "ssr" with BuildingType::Ssr and "ssg" with BuildingType::Ssg, ensuring accurate parsing of building types.

* test: add unit tests for API request handling and live reload functionality

This commit introduces a series of unit tests across multiple modules, including the API handler, server handler, live reload, and rebuilder. The tests cover scenarios such as missing or empty API directories, error handling for missing methods, and the injection of live reload scripts into HTML responses. These additions enhance the test coverage and ensure the reliability of the respective functionalities.

* refactor: improve formatting and readability in pages_generator and handler tests

This commit enhances the formatting of error messages in `pages_generator.rs` and improves the readability of test code in `handler.rs` and `live_reload.rs` by breaking long lines into multiple lines. These changes aim to maintain consistency and clarity throughout the codebase.

* feat: add metacall-sys as a build dependency and create build script for metassr-api-handler

This commit introduces the `metacall-sys` crate as a build dependency in the `Cargo.toml` of the `metassr-api-handler`. Additionally, a new `build.rs` file is created to facilitate the build process by invoking `metacall_sys::build()`. These changes enhance the build configuration for the API handler.

* feat: add metacall-sys to dependencies and implement build script for metassr-server

This commit adds the `metacall-sys` crate as a build dependency in the `Cargo.toml` of the `metassr-server` and introduces a new `build.rs` file that calls `metacall_sys::build()`. These changes enhance the build configuration for the server component.

* feat: add tempfile as a development dependency and refactor tests to use temporary directories

This commit introduces the `tempfile` crate as a development dependency in the `Cargo.toml` files of both `metassr-api-handler` and `metassr-server`. Additionally, it refactors the test cases in `lib.rs` and `handler.rs` to utilize temporary directories created by `tempfile`, improving test isolation and reliability.

* feat: add tempfile dependency to Cargo.lock and clean up test module imports

This commit adds the `tempfile` crate to the `Cargo.lock` file, ensuring it is available for development. Additionally, it removes unnecessary imports in the test module of `client/mod.rs`, streamlining the code for better readability.

* fix: ensure mutable routes in API handler test

This commit modifies the test for the API handler to declare `routes` as mutable. This change is necessary for the test to function correctly, as it prepares the `ApiRoutes` instance for potential modifications during the test execution.

* Update Cargo.toml

* Update Cargo.toml

---------

Co-authored-by: Mayank <mayank.jha@sellergeni.com>
Co-authored-by: Vicente Eduardo Ferrer Garcia <7854099+viferga@users.noreply.github.com>
- Adding test for HTML in `integration.yml` (#108)

* testing

* testing

* testing

* added debug

* added debug

* removed debug librabry

* added env

* added env

* added env

* updated install path

* updated install path

* fixed integration tests check

* fixed integration tests check

* testing by removing the --debug form install script

* added fix for npm permissions

* testing playwright

* testing playwright

* testing playwright

* testing playwright

* testing playwright

* testing playwright

* testing playwright

* html

* html

* correct html

* fixed play-wright for windows and added a new file

* commiting to fix back ubunyu and mac

* testing by adding bash if windows setup passes

* testing by adding bash if windows setup passes

* updated code with suggestions

* updated code with suggestions

* improved tests for HTML

* improved tests for HTML
- Rename "run" to "start:ssr" and correct Dockerfile CMD syntax (#121)
- Nix flakes unix support (#144)

* adding rust-toolchain

* add: new systems and update metacallConfig with each libpath and dyn path

* reference rust toolchain.toml to have consistent builds according to toolchain

* update shell hook to get the correct paths from metacallConfog

* add checks to verify nix builds in different systems

* remove targets file as thats taken care in nix files

* lock file

* added CI testing to test this

* fix CI test

* remvoe all-ssystmes

* removed install rust: since we have a rust toolchain, we dont need to install rust separately in actions

* Revert "removed install rust: since we have a rust toolchain, we dont need to install rust separately in actions"

This reverts commit 97fc458e518fccf134e6373c7baf4060edb9a9d4.
- Add unit tests for metassr-cli (#149)

* fix: correct BuildingType mapping for SSR and SSG in builder.rs

This commit fixes the mapping of string representations to the BuildingType enum. The previous implementation incorrectly mapped "ssr" to Ssg and "ssg" to Ssr. The corrected mappings now properly associate "ssr" with BuildingType::Ssr and "ssg" with BuildingType::Ssg, ensuring accurate parsing of building types.

* refactor: remove unused CLI builder and creator modules and document GSoC issues in project documentation

* cargo: format fix

---------

Co-authored-by: Mayank <mayank.jha@sellergeni.com>
- Is_rebuilding flag stuck after failed rebuild (#147)
- Test/live reload and watcher (#142)

* fix: correct BuildingType mapping for SSR and SSG in builder.rs

This commit fixes the mapping of string representations to the BuildingType enum. The previous implementation incorrectly mapped "ssr" to Ssg and "ssg" to Ssr. The corrected mappings now properly associate "ssr" with BuildingType::Ssr and "ssg" with BuildingType::Ssg, ensuring accurate parsing of building types.

* Add tests for live reload functionality and message serialization

- Introduced new tests in `live_reload.rs` to verify the injection of live reload scripts into HTML responses, ensuring that scripts are not injected when the body tag is missing and that original status codes are preserved.
- Added tests for message serialization in `utils.rs`, confirming that page and non-page messages serialize correctly with the expected structure.
- Enhanced the WebSocket server test to validate that rebuild messages are sent correctly over the WebSocket connection.

* Refactor test event creation for improved readability in `utils.rs`

- Updated the formatting of the `make_event` function call in the `ignores_metadata_modify_events` test to enhance clarity by breaking the line into multiple lines.

---------

Co-authored-by: Mayank <mayank.jha@sellergeni.com>
- Fixed --debug-mode to show logs (#153)

* take precedence from env to determine the logging level

* lint

* moving filter initialization to one layer over, as we init the same thing in both arms

* init tracing level for dynamic filtering

* pass http_debug flag to dev

* init http_debug flag in dev

* incorrect syntax for debug!()

* lint

* extract out a fn for building filter

* add tests for building filter

* lint

* borrow instead of move
- Optimize metassr-bundler (#119)

* refactor: vendored rspack

* optimize: disbale some bundling configurations for build

* optimize: combine client and server bundling

* optimize: replace rspack with esbuild

* fix: rustfmt

* fix: clippy

* fix the integration test

* improve the integration test

* fix

* fix vendored pacakges on windows

* remove spell check

* fix the ci
- Finish metassr-api-handler with only GET & POST support for javascript (#140)
- Random users app (#160)
- Only increment port when in dev mode (#166)
- Rebuilder module (#168)
- Replace some rspack mentions with esbuild (#169)
- Remove api hot-reload from rebuilder module
- Introduce Polyglot langauge api handlers to MetaSSR (#172)

* add multiple language Tags in Api handler

* failing: added python and ruby scripts to example API handler but it fails.

* fix failing test by silently ignoring other file extensions other than JS

* fix: loading scripts other that nodejs scripts

* remove rbgreet due to when loading ruby scripts metacall segfaults

* gitignore: add __pycache__ to gitignore

* remove pygreet due to it breaking the windows CI
- Bump nodejs version 20 -> 24
- Add python pygreet API Route (#173)
- *(`metassr-api-handler` tests)* Make them async tokio tests (#175)
- Introduce new crate: metassr-config (#179)

* introduce new crate: metassr-config

* remove todo macros

* load config function. supress not-read bindings to keep clippy temporary happy
- *(metassr-config)* Add server port under dev settings (#181)
- *(metassr-cli)* Rename `metassr run` command to `metassr start` (#182)
- Init sales-dashboard example (#183)

### Ignored

- *(deps)* Bump `rspack` to 1.5.5 (#57)
- *(deps)* Update all metacall crates (#70)

