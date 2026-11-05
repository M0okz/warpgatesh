"""Scoped browser checks; requires Python Playwright and Chromium (QA only)."""
import argparse
import json
from pathlib import Path
from urllib.parse import urljoin

from playwright.sync_api import expect, sync_playwright

parser = argparse.ArgumentParser()
parser.add_argument('--url', default='http://127.0.0.1:4176/')
parser.add_argument('--output', required=True)
args = parser.parse_args()
out = Path(args.output)
out.mkdir(parents=True, exist_ok=True)
checks, errors, pages = [], [], []


def check(ok, label):
    checks.append({'check': label, 'passed': bool(ok)})


routes = ['', 'docs/', *[f'docs/{slug}.html' for slug in (
    'desktop-app', 'installation', 'installation-linux', 'getting-started',
    'troubleshooting', 'uninstall')]]
with sync_playwright() as p:
    browser = p.chromium.launch(headless=True)
    context = browser.new_context(permissions=['clipboard-read', 'clipboard-write'])
    page = context.new_page()
    page.on('pageerror', lambda error: errors.append(str(error)))
    for width, height in [(320, 900), (375, 900), (768, 1024), (1440, 1000)]:
        page.set_viewport_size({'width': width, 'height': height})
        for route in routes:
            response = page.goto(urljoin(args.url, route), wait_until='networkidle')
            metrics = page.evaluate('''() => ({width: innerWidth,
                content: document.documentElement.scrollWidth,
                imagesReady: [...document.images].every(i => i.loading === 'lazy' || (i.complete && i.naturalWidth > 0)),
                mainTarget: document.querySelector('main').tabIndex === -1})''')
            pages.append({'route': route or '/', 'viewport': width, **metrics})
            check(response.status == 200, f'{width}: {route or "/"} responds')
            check(metrics['content'] <= width, f'{width}: {route or "/"} fits viewport')
            check(metrics['imagesReady'], f'{width}: {route or "/"} has its initial images')
            check(metrics['mainTarget'], f'{width}: {route or "/"} skip target can receive focus')
            if not route and width in [375, 768, 1440]:
                page.screenshot(path=str(out / f'home-{width}.png'))
                page.locator('#cli').screenshot(path=str(out / f'cli-{width}.png'))
                if width == 768:
                    page.locator('#companion').screenshot(path=str(out / 'companion-768.png'))
            if route == 'docs/installation.html' and width == 375:
                page.locator('.docs-content').screenshot(path=str(out / 'guide-375.png'))

    page.set_viewport_size({'width': 375, 'height': 900})
    page.goto(args.url, wait_until='networkidle')
    page.keyboard.press('Tab')
    expect(page.locator('.skip')).to_be_focused()
    page.keyboard.press('Enter')
    expect(page.locator('main')).to_be_focused()
    check(True, 'Skip link moves keyboard focus into main')
    menu, nav = page.locator('[data-menu]'), page.locator('#navigation')
    menu.focus()
    page.keyboard.press('Enter')
    expect(nav).to_be_visible()
    page.screenshot(path=str(out / 'mobile-menu.png'))
    page.keyboard.press('Escape')
    expect(menu).to_be_focused()
    expect(nav).to_be_hidden()
    check(True, 'Mobile menu opens by keyboard; Escape closes and restores focus')
    menu.click()
    page.get_by_role('link', name='The CLI', exact=True).click()
    expect(page.locator('#cli')).to_be_focused()
    expect(nav).to_be_hidden()
    check(True, 'Mobile section link closes menu and focuses its destination')
    menu.click()
    page.locator('.tools > p').first.click()
    expect(menu).to_have_attribute('aria-expanded', 'false')
    check(True, 'Outside click dismisses mobile menu')
    menu.click()
    page.set_viewport_size({'width': 1440, 'height': 1000})
    expect(menu).to_have_attribute('aria-expanded', 'false')
    page.set_viewport_size({'width': 375, 'height': 900})
    expect(nav).to_be_hidden()
    check(True, 'Resizing clears stale mobile menu state')

    buttons = page.locator('[data-copy]')
    for button in buttons.all():
        box = button.bounding_box()
        check(box['width'] >= 44 and box['height'] >= 44, f'{button.get_attribute("data-copy")}: touch target >= 44px')
    button = page.locator('[data-copy="cmd-sync"]')
    before = button.bounding_box()
    button.focus()
    page.keyboard.press('Space')
    expect(button).to_have_text('Copied')
    check(page.evaluate('navigator.clipboard.readText()') == 'warpgatesh sync', 'Copy writes the exact command')
    check(button.is_visible() and page.locator('.copy-status').inner_text() == 'Command copied to clipboard.', 'Visible Copy confirmation also has a live announcement')
    check(abs(button.bounding_box()['width'] - before['width']) < 0.1, 'Copy confirmation keeps button width stable')
    check(button.evaluate('b => getComputedStyle(b).transitionDuration') == '0s', 'Keyboard copy feedback has no animated transition')
    page.screenshot(path=str(out / 'copy-success.png'))
    page.wait_for_timeout(1300)
    button.click()
    expect(button).to_have_text('Copied')
    page.wait_for_timeout(900)
    expect(button).to_have_text('Copied')
    page.wait_for_timeout(1200)
    expect(button).to_have_text('Copy')
    check(True, 'Repeated copy restarts confirmation instead of letting an old timer clear it')
    page.evaluate('''Object.defineProperty(navigator.clipboard, 'writeText', {
        configurable: true, value: () => Promise.reject(new Error('QA denied clipboard'))})''')
    button.click()
    expect(button).to_have_text('Retry')
    expect(page.locator('.copy-status')).to_contain_text('Select the command')
    expect(button.locator('..').locator('.copy-error')).to_contain_text('Select the command')
    button.locator('..').screenshot(path=str(out / 'copy-denied.png'))
    check(True, 'Clipboard denial provides visible retry and manual-copy guidance')

    page.reload(wait_until='networkidle')
    summary = page.locator('.faq summary').first
    summary.focus()
    page.keyboard.press('Enter')
    expect(page.locator('.faq details').first).to_have_attribute('open', '')
    check(True, 'FAQ opens with Enter')
    page.emulate_media(reduced_motion='reduce')
    page.locator('[data-copy="cmd-sync"]').hover()
    check(page.locator('[data-copy="cmd-sync"]').evaluate('b => getComputedStyle(b).transitionDuration') == '0s', 'Reduced motion removes button transitions')
    check(page.evaluate('getComputedStyle(document.documentElement).scrollBehavior') == 'auto', 'Anchor navigation remains immediate')
    page.screenshot(path=str(out / 'reduced-motion.png'))
    page.close()

    plain = browser.new_context(java_script_enabled=False, viewport={'width': 375, 'height': 900})
    page = plain.new_page()
    page.goto(args.url, wait_until='networkidle')
    check(page.locator('#navigation').is_visible(), 'Mobile navigation remains available without JavaScript')
    check(not page.locator('[data-menu]').is_visible(), 'No inactive menu toggle without JavaScript')
    check(not page.locator('[data-copy]').first.is_visible(), 'No inactive copy buttons without JavaScript')
    page.screenshot(path=str(out / 'no-javascript.png'))
    browser.close()

check(not errors, 'No JavaScript runtime errors')
report = {'url': args.url, 'pages': pages, 'checks': checks, 'runtimeErrors': errors}
(out / 'browser-checks.json').write_text(json.dumps(report, indent=2) + '\n')
failed = [c['check'] for c in checks if not c['passed']]
print(json.dumps({'passed': len(checks) - len(failed), 'failed': failed, 'runtimeErrors': errors}, indent=2))
raise SystemExit(1 if failed else 0)
