#!/usr/bin/env node
const fs = require("fs");
const path = require("path");

const ROOT = path.resolve(__dirname, "..");
const OPEN_SOURCE_DIRS = ["uc-deliveries", "uc-fsr", "uc-hospital", "uc-lessons"];

function readText(filePath) {
  return fs.readFileSync(filePath, "utf8");
}

function verifyMvcPattern(staticDir) {
  const appJsPath = path.join(staticDir, "app.js");
  const mainMjsPath = path.join(staticDir, "app", "main.mjs");

  if (!fs.existsSync(appJsPath)) {
    throw new Error(`${staticDir}: missing static/app.js (MVC pattern required)`);
  }
  if (!fs.existsSync(mainMjsPath)) {
    throw new Error(`${staticDir}: missing static/app/main.mjs (MVC pattern required)`);
  }

  const indexPath = path.join(staticDir, "index.html");
  const modelsDir = path.join(staticDir, "app", "models");
  const uiDir = path.join(staticDir, "app", "ui");

  const appJsContent = readText(appJsPath);
  if (!appJsContent.includes("import { boot } from './app/main.mjs'")) {
    throw new Error(`${staticDir}: static/app.js must import boot from './app/main.mjs'`);
  }
  if (!appJsContent.includes("boot();")) {
    throw new Error(`${staticDir}: static/app.js must call boot()`);
  }

  if (!fs.existsSync(indexPath)) {
    throw new Error(`${staticDir}: missing static/index.html`);
  }

  const indexContent = readText(indexPath);
  if (!indexContent.includes('src="/app.js"')) {
    throw new Error(`${staticDir}: static/index.html must load /app.js`);
  }
  if (!indexContent.includes('type="module" src="/app.js"')) {
    throw new Error(`${staticDir}: static/index.html must use type="module" for /app.js`);
  }

  const mainMjsContent = readText(mainMjsPath);
  if (!mainMjsContent.includes("export") || !mainMjsContent.includes("boot")) {
    throw new Error(`${staticDir}: static/app/main.mjs must export boot function`);
  }

  if (!fs.existsSync(modelsDir)) {
    throw new Error(`${staticDir}: missing static/app/models/`);
  }
  if (!fs.existsSync(uiDir)) {
    throw new Error(`${staticDir}: missing static/app/ui/`);
  }

  return true;
}

function verifyUcDir(ucDir) {
  const staticDir = path.join(ROOT, ucDir, "static");
  if (!fs.existsSync(staticDir)) {
    throw new Error(`${ucDir}: missing static directory`);
  }

  verifyMvcPattern(staticDir);
  console.log(`${ucDir}: static MVC pattern verified.`);
}

function main() {
  for (const ucDir of OPEN_SOURCE_DIRS) {
    verifyUcDir(ucDir);
  }
  console.log("All use-case static patterns verified.");
}

try {
  main();
} catch (error) {
  console.error(error.message);
  process.exit(1);
}
