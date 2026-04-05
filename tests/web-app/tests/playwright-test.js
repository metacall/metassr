const { chromium } = require('playwright');

const BASE_URL = 'http://localhost:8080';

let browser;
let passed = 0;
let failed = 0;

async function assert(condition, message) {
    if (condition) {
        console.log(`  [PASS] ${message}`);
        passed++;
    } else {
        console.error(`  [FAIL] ${message}`);
        failed++;
    }
}

async function runTests() {
    console.log('Launching browser...');
    browser = await chromium.launch();

    await testIndexPage();
    await testHomePage();
    await testBlogPage();
    await testNavigation();
    await testFooterCounter();
    await testHomeCounter();
    await testApiEndpoint();

    await browser.close();

    console.log(`\n=== Results: ${passed} passed, ${failed} failed ===`);
    if (failed > 0) process.exit(1);
}

// --- Index page: static structure + SSR content ---
async function testIndexPage() {
    console.log('\n[Suite] Index page (/)');
    const page = await browser.newPage();
    await page.goto(BASE_URL);
    const html = await page.content();

    // Normalize backslashes to forward slashes for cross-platform path checks
    const normalizedHtml = html.replace(/\\/g, '/');

    await assert(html.includes('<title>My website</title>'), 'Has correct <title>');
    await assert(html.includes('lang="en"'), 'Has lang="en" attribute');
    await assert(html.includes('id="root"'), 'Has #root element');
    await assert(
        normalizedHtml.includes('/dist/pages/index.js.js'),
        'Has page JS bundle reference'
    );
    await assert(
        normalizedHtml.includes('/dist/pages/index.js.css'),
        'Has page CSS bundle reference'
    );

    // SSR-rendered dynamic content
    await assert(
        html.includes('Hello from index page'),
        'SSR: index page content rendered'
    );
    await assert(html.includes('href="/home"'), 'SSR: nav link to /home present');
    await assert(html.includes('href="/blog"'), 'SSR: nav link to /blog present');
    await assert(html.includes('This is a footer'), 'SSR: footer content rendered');

    await page.close();
}

// --- Home page: SSR content + initial counter state ---
async function testHomePage() {
    console.log('\n[Suite] Home page (/home)');
    const page = await browser.newPage();
    await page.goto(`${BASE_URL}/home`);
    const html = await page.content();

    await assert(
        html.includes('This is a simple home page contains a counter'),
        'SSR: home page heading rendered'
    );
    // Counter initial value should be 0 in SSR output
    await assert(
        html.includes('>0<') || html.includes('>0 <'),
        'SSR: counter initial value is 0'
    );

    await page.close();
}

// --- Blog page: SSR content ---
async function testBlogPage() {
    console.log('\n[Suite] Blog page (/blog)');
    const page = await browser.newPage();
    await page.goto(`${BASE_URL}/blog`);
    const html = await page.content();

    await assert(
        html.includes('This is a cool blog'),
        'SSR: blog page content rendered'
    );

    await page.close();
}

// --- Navigation: clicking header links changes content ---
async function testNavigation() {
    console.log('\n[Suite] Navigation');
    const page = await browser.newPage();
    await page.goto(BASE_URL);

    // Click "home" nav link
    await page.click('a[href="/home"]');
    await page.waitForLoadState('networkidle');
    const homeHtml = await page.content();
    await assert(
        homeHtml.includes('This is a simple home page contains a counter'),
        'Navigation: clicking /home renders home page content'
    );

    // Click "blog" nav link
    await page.click('a[href="/blog"]');
    await page.waitForLoadState('networkidle');
    const blogHtml = await page.content();
    await assert(
        blogHtml.includes('This is a cool blog'),
        'Navigation: clicking /blog renders blog page content'
    );

    // Click "index" nav link back
    await page.click('a[href="/"]');
    await page.waitForLoadState('networkidle');
    const indexHtml = await page.content();
    await assert(
        indexHtml.includes('Hello from index page'),
        'Navigation: clicking / returns to index page content'
    );

    await page.close();
}

// --- Footer counter: client-side interactivity ---
async function testFooterCounter() {
    console.log('\n[Suite] Footer counter (client-side interactivity)');
    const page = await browser.newPage();
    await page.goto(BASE_URL);

    const footerButton = page.locator('footer button');
    const initialText = await footerButton.innerText();
    await assert(
        initialText.includes('0'),
        `Footer counter starts at 0 (got: "${initialText.trim()}")`
    );

    await footerButton.click();
    const afterOneClick = await footerButton.innerText();
    await assert(
        afterOneClick.includes('1'),
        `Footer counter increments to 1 after click (got: "${afterOneClick.trim()}")`
    );

    await footerButton.click();
    await footerButton.click();
    const afterThreeClicks = await footerButton.innerText();
    await assert(
        afterThreeClicks.includes('3'),
        `Footer counter increments to 3 after 3 clicks (got: "${afterThreeClicks.trim()}")`
    );

    await page.close();
}

// --- Home counter: client-side interactivity ---
async function testHomeCounter() {
    console.log('\n[Suite] Home page counter (client-side interactivity)');
    const page = await browser.newPage();
    await page.goto(`${BASE_URL}/home`);

    const counterEl = page.locator('h1').filter({ hasText: /^\d+$/ });
    const initialValue = await counterEl.innerText();
    await assert(initialValue.trim() === '0', `Home counter starts at 0 (got: "${initialValue.trim()}")`);

    const clickButton = page.getByRole('button', { name: /click me/i });
    await clickButton.click();
    const afterClick = await counterEl.innerText();
    await assert(
        afterClick.trim() === '1',
        `Home counter increments to 1 after click (got: "${afterClick.trim()}")`
    );

    await page.close();
}

// --- API endpoint: dynamic JSON data ---
async function testApiEndpoint() {
    console.log('\n[Suite] API endpoint (/api/hello)');
    const page = await browser.newPage();

    // GET request
    const getResponse = await page.request.get(`${BASE_URL}/api/hello`);
    await assert(getResponse.ok(), `GET /api/hello returns 2xx status (got: ${getResponse.status()})`);

    const getBody = await getResponse.json().catch(() => null);
    if (getBody) {
        await assert(
            getBody.message === 'Hello from MetaSSR API!',
            `GET /api/hello body has correct message (got: "${getBody.message}")`
        );
        await assert(
            typeof getBody.timestamp === 'string' && getBody.timestamp.length > 0,
            'GET /api/hello body has dynamic timestamp field'
        );
    } else {
        // If response wraps body in a status envelope
        const raw = await getResponse.text();
        await assert(
            raw.includes('Hello from MetaSSR API!'),
            `GET /api/hello response includes expected message`
        );
    }

    // POST request with dynamic name
    const postResponse = await page.request.post(`${BASE_URL}/api/hello`, {
        headers: { 'Content-Type': 'application/json' },
        data: JSON.stringify({ name: 'MetaSSR' }),
    });
    await assert(postResponse.ok(), `POST /api/hello returns 2xx status (got: ${postResponse.status()})`);

    const postText = await postResponse.text();
    await assert(
        postText.includes('MetaSSR'),
        `POST /api/hello response echoes dynamic name "MetaSSR" (got: "${postText}")`
    );

    await page.close();
}

runTests().catch((err) => {
    console.error('Unexpected error:', err);
    if (browser) browser.close();
    process.exit(1);
});
