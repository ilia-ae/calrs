/* calrs embed runtime — pasted into a host site and exposes window.Calrs.
 *
 * API:
 *   Calrs.inline({ selector, link, config })
 *   Calrs.floatingButton({ link, buttonText, buttonPosition, buttonColor, textColor, showIcon, config })
 *   Calrs.elementClick()
 *   Calrs.autoInline()
 *
 * Where `link` is a full URL to a calrs booking page (e.g.
 * "https://cal.example.com/u/alice/intro") and `config` is an object with
 * optional `layout` ("month" | "week" | "column"), `theme` ("auto" | "light"
 * | "dark"), and `brand` (hex color, with or without leading "#"). These end
 * up as query parameters on the iframe src and feed into the embed-mode
 * rendering on the calrs side.
 *
 * ilia fork: no inline script needed. On load, every element carrying
 * `data-calrs-inline="<booking link>"` (and optionally `data-calrs-config`,
 * the same JSON as for data-calrs-link) gets the inline widget, replacing
 * the element's content, so a fallback link inside it shows only without
 * JavaScript. A host page whose CSP has no 'unsafe-inline' can then embed
 * with one <script src> alone. Links that are not http(s) are not loaded.
 *
 * ilia fork: the widget speaks the host page's language. `config.lang` (a
 * calrs language code such as "ru" or "de-formal"; "" for the visitor's
 * browser language) wins; otherwise the primary subtag of the page's
 * <html lang> ("ru-RU" -> "ru") is sent as `lang`. calrs uses it only when
 * it ships that language.
 *
 * ilia fork: the inline widget takes the host page's theme. Without
 * `config.theme`, the background behind the widget decides ("light" or
 * "dark" by its luminance; a page with none is light unless its root's
 * color-scheme is only dark), it is sent as `theme`, and a later change (a
 * site's own dark-mode switch, the system theme) is passed to the widget with
 * a `calrs:theme` message, so it switches in place. `config.theme` "light" or
 * "dark" fixes it; "auto" follows the visitor's system, as upstream does. The
 * iframe takes the same color-scheme (inline, `important`), since a scheme
 * different from the page inside makes the browser paint the frame opaque.
 */
(function () {
  if (window.Calrs) return;

  // ilia fork: the language to ask calrs for (see the header comment).
  function pageLang() {
    var l = (document.documentElement.getAttribute('lang') || '').toLowerCase().split(/[-_]/)[0];
    return /^[a-z]{2,3}$/.test(l) ? l : '';
  }
  function langFor(config) {
    var l = config && typeof config.lang === 'string' ? config.lang.toLowerCase() : pageLang();
    return /^[a-z]{2,3}(-[a-z]{2,8})?$/.test(l) ? l : '';
  }

  // ilia fork: the host page's theme behind `el` (see the header comment).
  function hostTheme(el) {
    for (var n = el; n && n.nodeType === 1; n = n.parentElement) {
      var m = /rgba?\(\s*([\d.]+)[,\s]+([\d.]+)[,\s]+([\d.]+)(?:[,\s\/]+([\d.]+%?))?/.exec(getComputedStyle(n).backgroundColor || '');
      if (!m) continue;
      var a = m[4] === undefined ? 1 : (m[4].slice(-1) === '%' ? parseFloat(m[4]) / 100 : parseFloat(m[4]));
      if (a < 0.5) continue;
      var y = (0.299 * m[1] + 0.587 * m[2] + 0.114 * m[3]) / 255;
      return y < 0.5 ? 'dark' : 'light';
    }
    var cs = getComputedStyle(document.documentElement).colorScheme || '';
    return /dark/.test(cs) && !/light/.test(cs) ? 'dark' : 'light';
  }
  function fixedTheme(config) {
    var t = config && config.theme;
    return t === 'light' || t === 'dark' || t === 'auto' ? t : '';
  }
  function schemeFor(theme) {
    return theme === 'light' || theme === 'dark' ? theme : 'light dark';
  }

  function buildSrc(link, config, theme) {
    if (!link) return 'about:blank';
    // ilia fork: an iframe with a javascript: or data: src would run in (or
    // look like) the host page; only http(s) booking links are loaded.
    var parsed;
    try { parsed = new URL(link, location.href); } catch (e) { return 'about:blank'; }
    if (parsed.protocol !== 'https:' && parsed.protocol !== 'http:') return 'about:blank';
    // Drop any #fragment first: appending our query after it would fold the
    // params into the fragment, so embed=1/layout/theme/brand would be lost.
    var hashIdx = link.indexOf('#');
    if (hashIdx >= 0) link = link.slice(0, hashIdx);
    var url = link + (link.indexOf('?') >= 0 ? '&' : '?') + 'embed=1';
    theme = theme || (config && config.theme);
    if (theme) url += '&theme=' + encodeURIComponent(theme);
    if (config) {
      if (config.layout) url += '&layout=' + encodeURIComponent(config.layout);
      if (config.brand) {
        var b = String(config.brand).replace(/^#/, '');
        url += '&brand=' + encodeURIComponent(b);
      }
    }
    var lang = langFor(config);
    if (lang) url += '&lang=' + encodeURIComponent(lang);
    return url;
  }

  function readConfig(el) {
    var raw = el.getAttribute('data-calrs-config');
    var cfg = {};
    if (raw) { try { cfg = JSON.parse(raw); } catch (e) {} }
    return cfg;
  }

  function originOf(url) {
    try { return new URL(url, location.href).origin; } catch (e) { return null; }
  }

  // Auto-size: parent listens for {type:'calrs:resize', height} from the
  // booking page inside the iframe. The booking page emits this on load and
  // whenever its content height changes. We accept messages only from the
  // iframe's own origin and contentWindow to avoid trusting unrelated frames.
  function attachResize(iframe) {
    var iframeOrigin = originOf(iframe.src);
    window.addEventListener('message', function (ev) {
      if (!ev.data || ev.data.type !== 'calrs:resize') return;
      if (iframeOrigin && ev.origin !== iframeOrigin) return;
      if (ev.source !== iframe.contentWindow) return;
      var h = Math.max(60, Math.floor(ev.data.height || 0)) + 8;
      iframe.style.height = h + 'px';
    });
  }

  // ilia fork: pass a change of the host page's theme to the widget, which
  // switches in place (templates/base.html, embed mode). Sent again on every
  // load of the frame, since a page inside only hears it once it listens.
  function followTheme(target, iframe, theme) {
    var origin = originOf(iframe.src);
    if (!origin) return;
    function tell() {
      try { iframe.contentWindow.postMessage({ type: 'calrs:theme', theme: theme }, origin); } catch (e) {}
    }
    function check() {
      var now = hostTheme(target);
      if (now === theme) return;
      theme = now;
      iframe.style.setProperty('color-scheme', schemeFor(theme), 'important');
      tell();
    }
    iframe.addEventListener('load', tell);
    if (window.MutationObserver) {
      var mo = new MutationObserver(check);
      mo.observe(document.documentElement, { attributes: true });
      if (document.body) mo.observe(document.body, { attributes: true });
    }
    if (window.matchMedia) {
      var mq = matchMedia('(prefers-color-scheme: dark)');
      var later = function () { setTimeout(check, 50); };
      if (mq.addEventListener) mq.addEventListener('change', later);
      else if (mq.addListener) mq.addListener(later);
    }
  }

  // Single shared modal — reused by floatingButton and elementClick.
  var modal = null;
  // Guards elementClick() against binding its document listener more than once.
  var elementClickBound = false;
  function ensureModal() {
    if (modal) return modal;
    var overlay = document.createElement('div');
    overlay.style.cssText =
      'position:fixed;inset:0;z-index:2147483647;background:rgba(0,0,0,0.55);' +
      'display:none;align-items:center;justify-content:center;padding:1rem;';
    var box = document.createElement('div');
    box.style.cssText =
      'background:#fff;border-radius:12px;width:min(960px,100%);height:min(90vh,720px);' +
      'overflow:hidden;position:relative;display:flex;flex-direction:column;' +
      'box-shadow:0 20px 60px rgba(0,0,0,0.4);';
    var close = document.createElement('button');
    close.type = 'button';
    close.setAttribute('aria-label', 'Close');
    close.innerHTML = '&times;';
    close.style.cssText =
      'position:absolute;top:0.5rem;right:0.5rem;width:34px;height:34px;border-radius:50%;' +
      'border:0;background:rgba(0,0,0,0.06);color:#000;font-size:22px;line-height:1;' +
      'cursor:pointer;z-index:2;';
    close.addEventListener('click', closeModal);
    var iframe = document.createElement('iframe');
    iframe.style.cssText = 'flex:1;width:100%;border:0;display:block;';
    iframe.setAttribute('allow', 'clipboard-write');
    iframe.setAttribute('title', 'Booking');
    box.appendChild(close);
    box.appendChild(iframe);
    overlay.appendChild(box);
    overlay.addEventListener('click', function (ev) {
      if (ev.target === overlay) closeModal();
    });
    document.addEventListener('keydown', function (ev) {
      if (ev.key === 'Escape' && modal && modal.overlay.style.display !== 'none') closeModal();
    });
    document.body.appendChild(overlay);
    modal = { overlay: overlay, iframe: iframe };
    return modal;
  }
  function openModal(link, config) {
    var m = ensureModal();
    m.iframe.src = buildSrc(link, config);
    m.overlay.style.display = 'flex';
    document.body.style.overflow = 'hidden';
  }
  function closeModal() {
    if (!modal) return;
    modal.overlay.style.display = 'none';
    modal.iframe.src = 'about:blank';
    document.body.style.overflow = '';
  }

  function calendarIcon() {
    var svg = document.createElementNS('http://www.w3.org/2000/svg', 'svg');
    svg.setAttribute('width', '18');
    svg.setAttribute('height', '18');
    svg.setAttribute('viewBox', '0 0 24 24');
    svg.setAttribute('fill', 'none');
    svg.setAttribute('stroke', 'currentColor');
    svg.setAttribute('stroke-width', '2');
    svg.setAttribute('stroke-linecap', 'round');
    svg.setAttribute('stroke-linejoin', 'round');
    svg.innerHTML =
      '<rect x="3" y="4" width="18" height="18" rx="2" ry="2"></rect>' +
      '<line x1="16" y1="2" x2="16" y2="6"></line>' +
      '<line x1="8" y1="2" x2="8" y2="6"></line>' +
      '<line x1="3" y1="10" x2="21" y2="10"></line>';
    return svg;
  }

  var Calrs = {
    inline: function (opts) {
      opts = opts || {};
      var target = typeof opts.selector === 'string' ? document.querySelector(opts.selector) : opts.selector;
      if (!target) return;
      var iframe = document.createElement('iframe');
      // ilia fork: the host page's theme unless the config fixes one.
      var fixed = fixedTheme(opts.config);
      var theme = fixed || hostTheme(target);
      iframe.src = buildSrc(opts.link, opts.config, theme);
      iframe.style.cssText = 'width:100%;min-height:560px;border:0;display:block;';
      iframe.style.setProperty('color-scheme', schemeFor(theme), 'important');
      iframe.setAttribute('loading', 'lazy');
      iframe.setAttribute('title', 'Booking');
      target.innerHTML = '';
      target.appendChild(iframe);
      attachResize(iframe);
      if (!fixed) followTheme(target, iframe, theme);
    },

    floatingButton: function (opts) {
      opts = opts || {};
      var btn = document.createElement('button');
      btn.type = 'button';
      var pos = opts.buttonPosition || 'bottom-right';
      var bg = opts.buttonColor || '#2563eb';
      var fg = opts.textColor || '#ffffff';
      var positions = {
        'bottom-right': 'bottom:1rem;right:1rem;',
        'bottom-left':  'bottom:1rem;left:1rem;',
        'top-right':    'top:1rem;right:1rem;',
        'top-left':     'top:1rem;left:1rem;'
      };
      btn.style.cssText =
        'position:fixed;' + (positions[pos] || positions['bottom-right']) +
        'z-index:2147483646;padding:0.75rem 1.25rem;border:0;border-radius:999px;' +
        'font-family:inherit;font-size:0.95rem;font-weight:600;cursor:pointer;' +
        'display:inline-flex;align-items:center;gap:0.5rem;' +
        'box-shadow:0 4px 14px rgba(0,0,0,0.15);' +
        'background:' + bg + ';color:' + fg + ';';
      if (opts.showIcon !== false) btn.appendChild(calendarIcon());
      var label = document.createElement('span');
      label.textContent = opts.buttonText || 'Book a meeting';
      btn.appendChild(label);
      btn.addEventListener('click', function () { openModal(opts.link, opts.config); });
      document.body.appendChild(btn);
    },

    elementClick: function () {
      // Idempotent: we auto-invoke on load AND expose this publicly, so a host
      // page calling it manually must not bind the document listener twice.
      if (elementClickBound) return;
      elementClickBound = true;
      document.addEventListener('click', function (ev) {
        var el = ev.target.closest('[data-calrs-link]');
        if (!el) return;
        ev.preventDefault();
        openModal(el.getAttribute('data-calrs-link'), readConfig(el));
      });
    },

    // ilia fork: inline widgets declared in markup (see the header comment).
    // Idempotent per element, so calling it again after adding markup only
    // fills the new ones.
    autoInline: function () {
      var els = document.querySelectorAll('[data-calrs-inline]');
      for (var i = 0; i < els.length; i++) {
        var el = els[i];
        if (el.getAttribute('data-calrs-ready') === '1') continue;
        el.setAttribute('data-calrs-ready', '1');
        Calrs.inline({ selector: el, link: el.getAttribute('data-calrs-inline'), config: readConfig(el) });
      }
    }
  };

  window.Calrs = Calrs;

  function start() {
    Calrs.elementClick();
    Calrs.autoInline();
  }
  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', start);
  } else {
    start();
  }
})();
