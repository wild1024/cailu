import { writeFileSync } from 'node:fs';
const BASE = 'http://127.0.0.1:5173';
const sleep = ms => new Promise(r => setTimeout(r, ms));
const targets = await (await fetch('http://127.0.0.1:9222/json')).json();
const page = targets.find(t => t.type === 'page') || targets[0];
const ws = new WebSocket(page.webSocketDebuggerUrl);
let msgId = 0; const pending = new Map();
function send(method, params = {}) { const id = ++msgId; return new Promise((res, rej) => { pending.set(id, { res, rej }); ws.send(JSON.stringify({ id, method, params })); }); }
ws.addEventListener('message', d => { const m = JSON.parse(d.data); if (m.id !== undefined && pending.has(m.id)) { const p = pending.get(m.id); pending.delete(m.id); m.error ? p.rej(m.error) : p.res(m.result); } });
ws.addEventListener('open', async () => {
  const ev = (expr) => send('Runtime.evaluate', { expression: expr, returnByValue: true });
  await send('Page.enable'); await send('Runtime.enable');
  await send('Page.navigate', { url: BASE + '/dashboard' }); await sleep(3000);

  // 1) force dark via the real toggle
  await ev("document.querySelector('.basic-layout__theme-toggle').click()");
  await sleep(800);
  const before = await ev("document.documentElement.getAttribute('theme-mode')");

  // 2) hard reload — initTheme() should restore dark from localStorage
  await send('Page.navigate', { url: BASE + '/dashboard' }); await sleep(3000);
  const after = await ev("(function(){return JSON.stringify({mode:document.documentElement.getAttribute('theme-mode'), saved:(JSON.parse(localStorage.getItem('cailu-setting')||'{}')).themeMode, bodyBg:getComputedStyle(document.body).backgroundColor, layout:!!document.querySelector('.basic-layout')});})()");
  console.log('before reload mode:', before.result.value);
  console.log('after reload:', after.result.value);
  const a = JSON.parse(after.result.value);
  const ok = a.mode === 'dark' && a.saved === 'dark' && a.layout === true;
  console.log(ok ? 'PERSIST-RELOAD: PASS' : 'PERSIST-RELOAD: FAIL');

  // restore light for a clean leave-behind
  await ev("document.querySelector('.basic-layout__theme-toggle').click()");
  await sleep(600);
  const fin = await ev("document.documentElement.getAttribute('theme-mode')");
  console.log('restored to:', fin.result.value);
  writeFileSync('theme-persist-result.json', JSON.stringify({ before: before.result.value, after: a, restored: fin.result.value, ok }, null, 2));
  ws.close();
});
