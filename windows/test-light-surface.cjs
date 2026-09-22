// Run with node --test test-light-surface.cjs from windows/.
// Keeps the two HTML windows on the same small contract: Settings persists a choice and the notch
// maps it to semantic CSS tokens instead of leaving a pale spinner on a pale surface.
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

test('light surface layers neutral grays with readable dark marks and restrained status colors', () => {
  const tokens = lightTokens(notch);
  for (const token of [
    '--surface:#f6f6f3;', '--card:#fff;', '--ink:#252522;', '--ink-strong:#161614;',
    '--edge:#c9c9c3;', '--track:#d8d8d2;', '--usage-ample:#0f704e;',
    '--usage-watch:#8a5a00;', '--usage-critical:#a83b32;'
  ]) {
    assert.ok(tokens.includes(token), `light surface is missing ${token}`);
  }
  assert.ok(!tokens.includes('--surface:#fff;'), 'the pill avoids a harsh pure-white field');
  assert.match(notch, /const neutral = name => getComputedStyle\(document\.body\)/,
    'runtime SVG rings read their neutral colour from the active surface');
  assert.match(notch, /const usageColor = name => neutral\(`usage-\$\{name\}`\)/,
    'status colors also follow the active surface');
  assert.match(notch, /listen\('notch_light_surface',e=>applyLightSurface\(e\.payload\)\)/,
    'the running notch reacts without a restart');
});

test('Appearance exposes and persists both named surfaces', () => {
  assert.match(settings, /id="seg-surface"[\s\S]*data-v="dark"[\s\S]*data-v="light"/);
  assert.match(settings, /invoke\('set_notch_light_surface', \{ on: lightSurface \}\)/);
  assert.match(settings, /'Surface':'Superfície'/);
  assert.match(settings, /'Light':'Clara'/);
});
