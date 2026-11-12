// ilia fork: passkeys and security keys (docs/passkeys-plan.md §4.9).
// Same-origin static file, no inline handlers, no fetch: a button runs the
// WebAuthn ceremony whose options the page embeds in
// <script type="application/json" id="webauthn-options">, fills the hidden
// fields of its form with the result (base64url) and submits the form as an
// ordinary POST. Every visible string is in the page (Fluent), not here.
//
// Markup it reads: a button[data-webauthn="get"|"create"] with data-form
// (the form's id) and, for create, data-hints ("security-key" or
// "client-device,hybrid"); the notices [data-webauthn-unsupported],
// [data-webauthn-cancelled], [data-webauthn-exists], [data-webauthn-key-type]
// (the browser cannot read the new key: Safari's getPublicKey() returns
// nothing for Ed25519 or RSA) and [data-webauthn-wrong-site] (SecurityError:
// this page is not at the address passkeys are bound to). Buttons start
// hidden, so a browser without JavaScript or WebAuthn never shows one.
(function () {
  'use strict';

  function b64urlToBytes(value) {
    var base64 = value.replace(/-/g, '+').replace(/_/g, '/');
    while (base64.length % 4) base64 += '=';
    var raw = atob(base64);
    var bytes = new Uint8Array(raw.length);
    for (var i = 0; i < raw.length; i++) bytes[i] = raw.charCodeAt(i);
    return bytes.buffer;
  }

  function bytesToB64url(buffer) {
    var bytes = new Uint8Array(buffer);
    var raw = '';
    for (var i = 0; i < bytes.length; i++) raw += String.fromCharCode(bytes[i]);
    return btoa(raw).replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/, '');
  }

  function decodeList(list) {
    return (list || []).map(function (c) {
      return { type: c.type, id: b64urlToBytes(c.id), transports: c.transports };
    });
  }

  function requestOptions(json) {
    if (typeof PublicKeyCredential.parseRequestOptionsFromJSON === 'function') {
      try { return PublicKeyCredential.parseRequestOptionsFromJSON(json); } catch (e) { /* decode below */ }
    }
    var o = Object.assign({}, json);
    o.challenge = b64urlToBytes(json.challenge);
    o.allowCredentials = decodeList(json.allowCredentials);
    return o;
  }

  function creationOptions(json) {
    if (typeof PublicKeyCredential.parseCreationOptionsFromJSON === 'function') {
      try { return PublicKeyCredential.parseCreationOptionsFromJSON(json); } catch (e) { /* decode below */ }
    }
    var o = Object.assign({}, json);
    o.challenge = b64urlToBytes(json.challenge);
    o.user = Object.assign({}, json.user, { id: b64urlToBytes(json.user.id) });
    o.excludeCredentials = decodeList(json.excludeCredentials);
    return o;
  }

  function show(selector, visible) {
    var nodes = document.querySelectorAll(selector);
    for (var i = 0; i < nodes.length; i++) nodes[i].hidden = !visible;
  }

  function setField(form, name, value) {
    var field = form.elements.namedItem(name);
    if (field) field.value = value;
  }

  function supported(kind) {
    if (!window.PublicKeyCredential || !navigator.credentials) return false;
    if (kind === 'create') {
      return typeof AuthenticatorAttestationResponse !== 'undefined' &&
        typeof AuthenticatorAttestationResponse.prototype.getPublicKey === 'function' &&
        typeof AuthenticatorAttestationResponse.prototype.getAuthenticatorData === 'function';
    }
    return true;
  }

  var NOTICES = ['cancelled', 'exists', 'key-type', 'wrong-site'];

  // Shows one notice. `reason` is 'key-type' when the browser handed over
  // no public key; else the error's name picks it. A retry cannot help with
  // the last two, so they do not say "try again".
  function failed(error, kind, reason) {
    for (var i = 0; i < NOTICES.length; i++) show('[data-webauthn-' + NOTICES[i] + ']', false);
    var name = error && error.name;
    var notice = 'cancelled';
    if (reason) {
      notice = reason;
    } else if (kind === 'create' && name === 'InvalidStateError') {
      notice = 'exists';
    } else if (name === 'SecurityError') {
      notice = 'wrong-site';
    }
    if (!document.querySelector('[data-webauthn-' + notice + ']')) notice = 'cancelled';
    show('[data-webauthn-' + notice + ']', true);
  }

  var busy = false;

  function setBusy(value) {
    busy = value;
    var buttons = document.querySelectorAll('button[data-webauthn]');
    for (var i = 0; i < buttons.length; i++) buttons[i].disabled = value;
  }

  function run(button, json) {
    if (busy) return;
    var kind = button.getAttribute('data-webauthn');
    var form = document.getElementById(button.getAttribute('data-form'));
    if (!form || !form.reportValidity()) return;
    var hints = (button.getAttribute('data-hints') || '').split(',').filter(Boolean);
    setBusy(true);
    var done = function () { setBusy(false); };
    var pending;
    try {
      pending = ceremony(kind, json, hints);
    } catch (e) {
      done();
      failed(e, kind);
      return;
    }
    pending.then(function (cred) {
      if (!cred) { done(); failed(null, kind); return; }
      if (!fill(form, kind, cred)) { done(); failed(null, kind, 'key-type'); return; }
      // submit() fires no submit event, so the handler below is not re-entered.
      form.submit();
    }, function (e) { done(); failed(e, kind); });
  }

  function ceremony(kind, json, hints) {
    if (kind === 'get') return navigator.credentials.get({ publicKey: requestOptions(json) });
    var withHints = Object.assign({}, json, hints.length ? { hints: hints } : {});
    var options = creationOptions(withHints);
    if (hints.length) options.hints = hints;
    return navigator.credentials.create({ publicKey: options });
  }

  function fill(form, kind, cred) {
    var r = cred.response;
    if (kind === 'get') {
      setField(form, 'credential_id', bytesToB64url(cred.rawId));
      setField(form, 'client_data', bytesToB64url(r.clientDataJSON));
      setField(form, 'authenticator_data', bytesToB64url(r.authenticatorData));
      setField(form, 'signature', bytesToB64url(r.signature));
      setField(form, 'user_handle', r.userHandle ? bytesToB64url(r.userHandle) : '');
      return true;
    }
    var key = r.getPublicKey();
    if (!key) return false;
    setField(form, 'credential_id', bytesToB64url(cred.rawId));
    setField(form, 'client_data', bytesToB64url(r.clientDataJSON));
    setField(form, 'authenticator_data', bytesToB64url(r.getAuthenticatorData()));
    setField(form, 'public_key', bytesToB64url(key));
    setField(form, 'algorithm', String(r.getPublicKeyAlgorithm()));
    setField(form, 'transports',
      JSON.stringify(typeof r.getTransports === 'function' ? r.getTransports() : []));
    return true;
  }

  // Enter in a field of a passkey form (the password on the confirm page,
  // the key name) would post it with the key fields empty, and the server
  // would count a wrong attempt: run the form's first key button instead,
  // or do nothing when the browser cannot use keys.
  function guardSubmit(form, json) {
    form.addEventListener('submit', function (event) {
      var filled = form.elements.namedItem('credential_id');
      if (filled && filled.value) return;
      event.preventDefault();
      var first = form.querySelector('button[data-webauthn]:not([hidden])') ||
        document.querySelector('button[data-webauthn][data-form="' + form.id + '"]:not([hidden])');
      if (first) run(first, json);
    });
  }

  function init() {
    var node = document.getElementById('webauthn-options');
    var buttons = document.querySelectorAll('button[data-webauthn]');
    if (!node || !buttons.length) return;
    var json;
    try { json = JSON.parse(node.textContent); } catch (e) { return; }
    for (var i = 0; i < buttons.length; i++) {
      var button = buttons[i];
      var form = document.getElementById(button.getAttribute('data-form'));
      if (form && !form.hasAttribute('data-webauthn-guarded')) {
        form.setAttribute('data-webauthn-guarded', '');
        guardSubmit(form, json);
      }
      if (!supported(button.getAttribute('data-webauthn'))) {
        button.hidden = true;
        show('[data-webauthn-unsupported]', true);
        continue;
      }
      button.hidden = false;
      button.addEventListener('click', function (event) {
        event.preventDefault();
        run(event.currentTarget, json);
      });
    }
  }

  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', init);
  } else {
    init();
  }
})();
