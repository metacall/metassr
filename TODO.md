## TODO

#### Main features

- [x] Serving static files are located in ``./static/**``

- [x] the HTML builder

> the HTML builder takes react pages and generate it to HTML pages to rendering it

- [x] Build the files loader

> it extract all files that locate in `/src` and categorize it (react pages, or special files (like [_head.jsx](./tests/web-app/src/_head.tsx), [_app.jsx](./tests/web-app/src/_app_.tsx))), and load it to metacall.

- [x] Serving ``./src/pages/**``

- [x] implement custom fallback page

- [ ] implement a node module for `metassr`
  - Hooks:
    - [ ] useProps
    - [ ] useParams
    - [ ] useHandlerResult

- [ ] Server handler
  
    A function executes in the server side when the client sends an http request.

    **Example**

    ```javascript
    import React, { useState, ReactNode } from 'react';
    import { useHandlerResult, usePageProps } from "metassr"
    export default function Article() {
        let [counter, setCounter] = useState(0);
        let { title } = usePageProps();
        let { data } = useHandlerResult();

        return (
            <div>
                <div className="text-4xl font-bold">This is a cool article</div>
                <div>Article's title: {title}</div>
            </div>
        )

    }


    export function serverHandler(req: Request): HandlerResult {
        let articles = read_article_content_from_db();
        // ... Stuff

        return {
            data: articles,
            statusCode: 200
            // ...
        }
    }
    ```

- [ ] ``api`` route.

- [x] `create` command for `metassr-cli`.

- [ ] Finish API Handler & add languages support
- [ ] dev mode
  - [x] websocket port now is hardcoded to `3001`
  - [ ] Granular Rebuilds
    - right now, the `rebuild_page()` function rebuilds both client and server bundles entirely, this is slow. only rebuild changed pages, and eventually, changed components
  - [ ] implement real HMR, not just restarting the page (line 34 in live_reload.js)
    - The page-level live reload works, but it's essentially just a "full reload on file change" system, not true hot-reloading
  - [ ] Layout Rebuild
  - [ ] Component Rebuild
  - [ ] Style Rebuild
  - [ ] Static Asset Rebuild
  - [ ] **Big Feature:** Add debugging tools inside dev-mode. like Vue js
    - [ ] Compile error overlay in browser (like Next.js/Vite)
      - Right now if a build fails, the user has to check the terminal. A red error overlay in the browser is a huge DX improvement.

- [ ] Proper Documentation for most of the crates
- [ ] Proper Documentation for how to deploy MetaSSR
- [ ] Middleware for Auth & Security
- [ ] Update templates
- [ ] add `metassr.config.js` to do everything the CLI does. Arguments sent in the CLI overrides the config.
  - [ ] custom port
  - [ ] http logging
  - [ ] middleware
  - [ ] output dir
  - [ ] images allowed URLs (like Nextjs)
  - [ ] override our opinionated Rspack config (?)

- [ ] examples for metassr polyglot usage
  - really really late-stage

- tests for:
  - [ ] live-reload
  - [ ] rebuilding
  - [ ] full server run via Docker
    - server starts and responds
    - 404 pages work
    - static files received
    - live reload

