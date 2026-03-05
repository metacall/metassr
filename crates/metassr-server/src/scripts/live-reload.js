/**
 * This script handles live reloading for MetaSSR development mode.
 * It establishes a WebSocket connection to receive rebuild notifications
 * and updates the page accordingly. It also renders a compile-error overlay
 * (similar to Next.js / Vite) when a build fails.
 */

(function() {
    let isReconnecting = false;
    let ws

	/// error overlay ///

	const OVERLAY_ID = '__metassr_error_overlay__';

	function showErrorOverlay(errors) {
		dismissErrorOverlay(); // remove any existing overlay first

		const overlay = document.createElement('div');
		overlay.id = OVERLAY_ID;
		overlay.style.cssText = [
			'position: fixed',
			'inset: 0',
			'z-index: 99999',
			'display: flex',
			'align-items: center',
			'justify-content: center',
			'background: rgba(0, 0, 0, 0.6)',
			'backdrop-filter: blur(4px)',
			'font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif',
			'padding: 24px',
			'box-sizing: border-box',
		].join(';');

		// remove existing overlay on backdrop click
		overlay.addEventListener('click', (e) => {
			if (e.target === overlay) dismissErrorOverlay();
		});

		const card = document.createElement('div');
		card.style.cssText = [
			'background: #ffffff',
			'box-shadow: 0 8px 32px rgba(0,0,0,0.2)',
			'border-left: 6px solid #ef4444',
			'border-radius: 8px',
			'max-width: 860px',
			'width: 100%',
			'max-height: 85vh',
			'overflow: auto',
			'padding: 32px 40px',
			'color: #111827',
			'font-size: 14px',
			'line-height: 1.6',
			'position: relative'
		].join(';');

		const header = document.createElement('div');
		header.style.cssText = [
			'display: flex',
			'align-items: center',
			'justify-content: space-between',
			'margin-bottom: 24px',
			'padding-bottom: 16px',
			'border-bottom: 1px solid #e5e7eb'
		].join(';');

		const title = document.createElement('h1');
		title.textContent = 'Unhandled Runtime Error';
		title.style.cssText = [
			'color: #111827',
			'font-size: 20px',
			'font-weight: 600',
			'margin: 0'
		].join(';');

		const closeBtn = document.createElement('button');
		closeBtn.innerHTML = '<svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="18" y1="6" x2="6" y2="18"></line><line x1="6" y1="6" x2="18" y2="18"></line></svg>';
		closeBtn.style.cssText = [
			'background: none',
			'border: none',
			'color: #6b7280',
			'cursor: pointer',
			'padding: 4px',
			'border-radius: 4px',
			'display: flex',
			'align-items: center',
			'justify-content: center'
		].join(';');
		closeBtn.onclick = dismissErrorOverlay;
		// hover effect for close button
		closeBtn.onmouseover = () => closeBtn.style.color = '#111827';
		closeBtn.onmouseout = () => closeBtn.style.color = '#6b7280';

		header.appendChild(title);
		header.appendChild(closeBtn);

		// subtitle indicating the type of error
		const subtitle = document.createElement('p');
		subtitle.textContent = 'Failed to compile';
		subtitle.style.cssText = [
			'color: #ef4444',
			'font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono", "Courier New", monospace',
			'font-size: 14px',
			'margin: 0 0 16px 0',
			'font-weight: 500'
		].join(';');

		card.appendChild(header);
		card.appendChild(subtitle);

		const errorList = Array.isArray(errors) ? errors : [errors];
		errorList.forEach((msg, index) => {
			const pre = document.createElement('pre');
			// strip ANSI escape codes from terminal output including Rust escaped format like \u{1b}[31m
			const cleanMessage = (msg || '').replace(/(\\u(?:\{1b}|001b)|[\u001b\u009b])[[\]()#;?]*(?:[0-9]{1,4}(?:;[0-9]{0,4})*)?[0-9A-ORZcf-nqry=><]/g, '');
			pre.textContent = cleanMessage;
			pre.style.cssText = [
				`margin: ${index === 0 ? '0' : '16px 0 0 0'}`,
				'white-space: pre-wrap',
				'word-break: break-word',
				'background: #111827',
				'color: #f3f4f6',
				'padding: 20px',
				'border-radius: 6px',
				'font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono", "Courier New", monospace',
				'font-size: 13px',
				'overflow-x: auto'
			].join(';');
			card.appendChild(pre);
		});

		const hint = document.createElement('p');
		hint.textContent = 'This error occurred during the build process. Fix the error to make this overlay disappear automatically or close it manually to continue.';
		hint.style.cssText = [
			'margin: 24px 0 0',
			'color: #6b7280',
			'font-size: 13px'
		].join(';');

		card.appendChild(hint);
		overlay.appendChild(card);

		document.body.appendChild(overlay);
	}

	function dismissErrorOverlay() {
		const existing = document.getElementById(OVERLAY_ID);
		if (existing) existing.remove();
	}

    /// websocket ///

    function connect() {
        if (ws) ws.close(); // Close old connection
        ws = new WebSocket('ws://localhost:__WS_PORT__')
        ws.onopen = () => {
            // signal to the server that the message handler is ready so it can
            // immediately push any cached build errors without an arbitrary delay.
            ws.send(JSON.stringify({ type: 'ready' }));
        };
        ws.onmessage = (event) => {
            const update = JSON.parse(event.data)
            const currentPath = window.location.pathname; //current page path

			switch (update.type) {
				case 'build_error':
					// Show the overlay with the compiler error messages
					showErrorOverlay(update.errors || ['Unknown build error']);
					break;
				case 'page':
					dismissErrorOverlay();
					if (update.path) {
					    reloadPage(update.path, currentPath);
					}
				    break;
				case 'layout':
					dismissErrorOverlay();
					window.location.reload();
				    break;
				
				case 'style':
					dismissErrorOverlay();
					reloadStylesheets();
					break
				case 'component':
					// this reloads the page anyways, this solution is temporary
					// Todo: make a function reloadComponent(update.path);
					dismissErrorOverlay();
					window.location.reload();
					break;
				case 'static':
					dismissErrorOverlay();
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

        return pathname === match || pathname === match + "/";
    }

    function reloadStylesheets() {
        const links = document.querySelectorAll('link[rel="stylesheet"]');
        links.forEach(link => {
            const href = link.href.split('?')[0];
            link.href = `$[href]?t=${Date.now()}`;
        })
    };
    // Start the live reload connection
    connect();
})();