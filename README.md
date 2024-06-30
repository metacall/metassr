> **Warning**  
> this project under development, **Don't use it in production**.

<div align="center">
 <h1>MetaSSR</h1>

<p align='center'> SSR framework built on <a href="https://github.com/metacall/core">MetaCall</a> </p>
</div>

## Running 
> We face issues currently working on
```terminal
$ cargo run --bin metassr-cli --root=tests/web-app
```



## TODO

#### Main features
- [ ] Serving staic files are located in ``./static/**``

- [ ] the HTML builder

> the HTML builder takes react pages and generate it to HTML pages to rendering it


- [ ] Build the files loader
 
> it extract all files that locate in `/src` and catagorize it (react pages, or special files (like [_head.jsx](./tests/web-app/src/_head.tsx), [_app.jsx](./tests/web-app/src/_app_.tsx))), and load it to metacall.

- [ ] Serving ``./src/pages/**``

- [ ] implement custom fallback page 

- [ ] Serving markdown files are located in `./static/**`
