## TODO

#### Main features
- [x] Serving staic files are located in ``./static/**``

- [x] the HTML builder

> the HTML builder takes react pages and generate it to HTML pages to rendering it


- [x] Build the files loader
 
> it extract all files that locate in `/src` and catagorize it (react pages, or special files (like [_head.jsx](./tests/web-app/src/_head.tsx), [_app.jsx](./tests/web-app/src/_app_.tsx))), and load it to metacall.

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

- [ ] `create` command for `metassr-cli`.