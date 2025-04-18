// scripts/gen-build-tag.js
import { writeFileSync } from 'node:fs';

const stamp = new Date().toISOString().slice(0,19);       // 2025‑04‑18T12:34:56
writeFileSync('.env.development', `PUBLIC_BUILD_VERSION=dev-${stamp}\n`);