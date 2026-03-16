/**
 * This script handles live reloading for MetaSSR development mode.
 * It establishes a WebSocket connection to receive rebuild notifications
 * and updates the page accordingly.
 */

(function () {
    let isReconnecting = false;
    let ws

    function connect() {
        if (ws) ws.close(); // Close old connection
        ws = new WebSocket('ws://localhost:__WS_PORT__')
        ws.onmessage = (event) => {
            const update = JSON.parse(event.data)
            const currentPath = window.location.pathname; //current page path

            switch (update.type) {
                case 'page':
                    if (update.path) {
                        reloadPage(update.path, currentPath);
                    }
                    break;
                case 'layout':
                    window.location.reload();
                    break;

                case 'style':
                    reloadStylesheets();
                    break
                case 'component':
                    // this reloads the page anyways, this solution is temporary 
                    // todo make a function reloadComponent(update.path);
                    window.location.reload();
                    break;
                case 'static':
                    // Reload the page to see static asset changes
                    window.location.reload();
                    break;
            }
        }

        ws.onclose = () => {
            if (!isReconnecting) {
                isReconnecting = true;
                setTimeout(() => {
                    isReconnecting = false;
                    connect();
                }, 1000);
            }
        }
    };

    function reloadPage(path, currentPath) {
        if (urlMatchPath(path, currentPath)) {
            window.location.reload();
        }
    }

    function urlMatchPath(filePath, pathname) {

        const match = filePath
            .replace(/^src\/pages/, "") // remove "src/pages"
            .replace(/\.(t|j)sx?$/, "") // remove extension
            .replace(/index$/, ""); // "index" files map to "/"

        return pathname == match || pathname == match + "/";
    }

    function reloadStylesheets() {
        const links = document.querySelectorAll('link[rel="stylesheet"]');
        links.forEach(link => {
            const href = link.href.split('?')[0];
            // Append a cache-busting timestamp to the stylesheet URL
            // This forces the browser to fetch the new CSS instead of using the stale cached version
            link.href = `${href}?t=${Date.now()}`;
        });
    };
    // Start the live reload connection
    connect();
})();