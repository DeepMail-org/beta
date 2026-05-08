/// Playwright Node.js script that runs inside the Docker container.
///
/// The container image has Playwright + Chromium pre-installed.
/// This script is passed via `node -e` to the container.

pub const PLAYWRIGHT_SCRIPT: &str = r##"
const playwright = require('playwright');
const fs = require('fs');

(async () => {
  const url = process.env.URL_TO_ANALYZE;
  const result = {
    final_url: null,
    title: null,
    redirect_chain: [],
    network_requests: [],
    cookies: [],
    has_password_field: false,
    has_email_field: false,
    has_login_form: false,
    has_download_trigger: false,
    external_scripts: [],
    iframes: [],
    page_html: null,
    js_dialogs: [],
    meta_description: null,
    error: null
  };

  let browser;
  try {
    browser = await playwright.chromium.launch({
      args: ['--no-sandbox', '--disable-setuid-sandbox', '--disable-dev-shm-usage']
    });

    const context = await browser.newContext({
      viewport: { width: 1280, height: 720 },
      userAgent: 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36'
    });

    const page = await context.newPage();

    // Track network requests
    const requests = [];
    page.on('request', req => {
      requests.push({
        url: req.url(),
        method: req.method(),
        resource_type: req.resourceType()
      });
    });

    page.on('response', resp => {
      const status = resp.status();
      if (status >= 300 && status < 400) {
        result.redirect_chain.push(resp.url());
      }
      // Update requests with status
      const idx = requests.findIndex(r => r.url === resp.url());
      if (idx >= 0) {
        requests[idx].status = status;
      }
      // Check for download triggers
      const cd = resp.headers()['content-disposition'];
      if (cd && cd.includes('attachment')) {
        result.has_download_trigger = true;
      }
    });

    // Track JS dialogs
    page.on('dialog', async dialog => {
      result.js_dialogs.push(dialog.message());
      await dialog.dismiss();
    });

    try {
      await page.goto(url, { timeout: 25000, waitUntil: 'networkidle' });
    } catch (navErr) {
      result.error = 'Navigation error: ' + navErr.message;
    }

    // Collect results even if navigation had issues
    result.final_url = page.url();
    result.title = await page.title().catch(() => null);
    result.network_requests = requests;

    // Cookies
    result.cookies = await context.cookies().catch(() => []);

    // DOM analysis
    try {
      result.has_password_field = (await page.$$('input[type=password]')).length > 0;
      result.has_email_field = (await page.$$('input[type=email]')).length > 0;

      // Login form: any form containing a password field
      result.has_login_form = await page.evaluate(() => {
        const forms = document.querySelectorAll('form');
        for (const f of forms) {
          if (f.querySelector('input[type=password]')) return true;
        }
        return false;
      }).catch(() => false);

      // External scripts
      const finalDomain = new URL(page.url()).hostname;
      const scripts = await page.$$eval('script[src]', els => els.map(e => e.src));
      result.external_scripts = scripts.filter(s => {
        try { return new URL(s).hostname !== finalDomain; } catch { return false; }
      });

      // Iframes
      result.iframes = await page.$$eval('iframe[src]', els => els.map(e => e.src));

      // Meta description
      result.meta_description = await page.$eval(
        'meta[name="description"]',
        el => el.content
      ).catch(() => null);

      // Page HTML (truncate to 50KB)
      const html = await page.content();
      result.page_html = html.length > 51200 ? html.slice(0, 51200) : html;
    } catch (domErr) {
      // DOM analysis errors are non-fatal
      if (!result.error) result.error = 'DOM analysis error: ' + domErr.message;
    }

    // Screenshot
    try {
      await page.screenshot({ path: '/results/screenshot.png', fullPage: false });
    } catch (ssErr) {
      // Non-fatal
    }

    await browser.close();
  } catch (err) {
    result.error = result.error || ('Fatal error: ' + err.message);
    if (browser) await browser.close().catch(() => {});
  }

  // Always write result
  fs.writeFileSync('/results/result.json', JSON.stringify(result));
  process.exit(0);
})();
"##;
