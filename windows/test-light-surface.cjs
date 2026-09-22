// Run with node --test test-light-surface.cjs from windows/.
// Keeps the two HTML windows on the same small contract: Settings persists a choice and the notch
// maps it to semantic CSS tokens instead of leaving a white spinner on a white surface.
const { readFileSync } = require('node:fs');
const { join } = require('node:path');
const assert = require('node:assert/strict');
const { test } = require('node:test');

const root = __dirname;
const notch = readFileSync(join(root, 'codenotch/ui/notch.html'), 'utf8');
const settings = readFileSync(join(root, 'codenotch/ui/settings.html'), 'utf8');

function lightTokens(source) {
  const match = source.match(/body\.light-surface\s*\{([^}]*)\}/);
  assert.ok(match, 'the notch has a light-surface token block');
  return match[1].replace(/\s/g, '');
}

test('light surface is opaque white with readable black neutral marks', () => {
  const tokens = lightTokens(notch);
  for (const token of ['--surface:#fff;', '--card:#fff;', '--ink:#111;', '--ink-strong:#111;', '--track:#1d1d1f;']) {
    assert.ok(tokens.includes(token), `light surface is missing ${token}`);
  }
  assert.match(notch, /const neutral = name => getComputedStyle\(document\.body\)/,
    'runtime SVG rings read their neutral colour from the active surface');
  assert.match(notch, /listen\('notch_light_surface',e=>applyLightSurface\(e\.payload\)\)/,
    'the running notch reacts without a restart');
});

test('Appearance exposes and persists both named surfaces', () => {
  assert.match(settings, /id="seg-surface"[\s\S]*data-v="dark"[\s\S]*data-v="light"/);
  assert.match(settings, /invoke\('set_notch_light_surface', \{ on: lightSurface \}\)/);
  assert.match(settings, /'Surface':'Superfície'/);
  assert.match(settings, /'Light':'Clara'/);
});
