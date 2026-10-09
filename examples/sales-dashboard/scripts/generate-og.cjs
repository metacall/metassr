#!/usr/bin/env node
const { mkdirSync, writeFileSync } = require('node:fs');
const { dirname, join } = require('node:path');
const { renderOG } = require('../src/api/og.js');

const out = join(__dirname, '..', 'static', 'og', 'dashboard.png');
mkdirSync(dirname(out), { recursive: true });
writeFileSync(out, renderOG('dashboard'));
console.log(`Wrote ${out}`);
