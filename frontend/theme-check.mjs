import { writeFileSync } from 'node:fs';
const BASE = 'http://127.0.0.1:5173';
const sleep = ms => new Promise(r => setTimeout(r, ms));

const targets = await (await fetch('http://127.0.0.1:9222/json')).json();
const page = targets.find(t => t.type === 'page') || targets[0];
const ws = new WebSocket(page.webSocketDebuggerUrl);
let msgId = 0; const pending = new Map();
function send(method, params = {}) {
  const id = ++msgId;
  return new Promise((res, rej) => { pending.set(id, { res, rej }); ws.send(JSON.stringify({ id, method, params })); });
}
const events = {};
ws.addEventListener('message', d => {
  const m = JSON.parse(d.data);
  if (m.id !== undefined && pending.has(m.id)) { const p = pending.get(m.id); pending.delete(m.id); m.error ? p.rej(m.error) : p.res(m.result); }
  else if (m.method) { (events[m.method] || []).forEach(cb => cb(m.params)); }
});
ws.addEventListener('open', async () => {
  await send('Page.enable'); await send('Runtime.enable'); await send('Log.enable');
  const errors = [];
  (events['Log.entryAdded'] = events['Log.entryAdded'] || []).push(p => { if (p.entry.level === 'error') errors.push('[console] ' + p.entry.text); });
  (events['Runtime.exceptionThrown'] = events['Runtime.exceptionThrown'] || []).push(p => { const d = p.exceptionDetails; errors.push('[pageerror] ' + (d.exception && d.exception.description || d.text)); });
  (events['Runtime.consoleAPICalled'] = events['Runtime.consoleAPICalled'] || []).push(p => { if (p.type === 'error') { const a = (p.args || []).map(x => x.value !== undefined ? x.value : (x.description || '')).join(' '); errors.push('[console.error] ' + a); } });

  const ev = (expr) => send('Runtime.evaluate', { expression: expr, returnByValue: true });
  async function navigate(path) { await send('Page.navigate', { url: BASE + path }); await sleep(2600); }
  async function setInput(sel, val) {
    return ev(`(function(){var el=document.querySelector(${JSON.stringify(sel)});if(!el)return 'NO_EL';var s=Object.getOwnPropertyDescriptor(window.HTMLInputElement.prototype,'value').set;s.call(el,${JSON.stringify(val)});el.dispatchEvent(new Event('input',{bubbles:true}));el.dispatchEvent(new Event('change',{bubbles:true}));return 'OK';})()`);
  }
  const click = (sel) => ev(`(function(){var el=document.querySelector(${JSON.stringify(sel)});if(!el)return 'NO_EL';el.click();return 'OK';})()`);
  async function snap(tag) {
    const r = await ev(`(function(){
      var cs=getComputedStyle(document.body);
      var root=getComputedStyle(document.documentElement);
      var aside=document.querySelector('.basic-layout__aside');
      var card=document.querySelector('.t-card');
      var btn=document.querySelector('.t-button--theme-primary');
      var header=document.querySelector('.basic-layout__header');
      return JSON.stringify({
        themeMode: document.documentElement.getAttribute('theme-mode'),
        togglePresent: !!document.querySelector('.basic-layout__theme-toggle'),
        localStorageTheme: (JSON.parse(localStorage.getItem('cailu-setting')||'{}')).themeMode || null,
        bodyBg: cs.backgroundColor,
        textColor: cs.color,
        brand7: root.getPropertyValue('--td-brand-color-7').trim(),
        bgPage: root.getPropertyValue('--td-bg-color-page').trim(),
        asideBg: aside?getComputedStyle(aside).backgroundColor:null,
        headerBg: header?getComputedStyle(header).backgroundColor:null,
        cardBg: card?getComputedStyle(card).backgroundColor:null,
        btnBg: btn?getComputedStyle(btn).backgroundColor:null,
        menuTheme: (function(){var m=document.querySelector('.basic-layout__menu');return m?m.getAttribute('theme'):null;})()
      });
    })()`);
    const data = JSON.parse(r.result.value);
    data.tag = tag;
    try {
      const shot = await send('Page.captureScreenshot', { format: 'png', captureBeyondViewport: false });
      const b64 = (shot && shot.data) ? shot.data : (shot && shot.result && shot.result.data);
      if (b64) {
        writeFileSync('theme-' + tag + '.png', Buffer.from(b64, 'base64'));
        data.screenshot = 'theme-' + tag + '.png';
      } else {
        data.screenshot = 'EMPTY';
      }
    } catch (e) {
      data.screenshot = 'ERR:' + String(e);
    }
    return data;
  }

  // ---- login ----
  await navigate('/');
  await ev("localStorage.clear();sessionStorage.clear();");
  await navigate('/login'); await sleep(1800);
  await setInput('input[placeholder="请输入用户名"]', 'admin');
  await setInput('input[placeholder="请输入密码"]', 'Admin@123456');
  await click('button[type=submit]');
  await sleep(4000);
  const afterLogin = await ev("(function(){return JSON.stringify({path:location.pathname, hasBasicLayout:!!document.querySelector('.basic-layout'), overlay:!!document.querySelector('vite-error-overlay,.vite-error-overlay'), bodyLen:(document.body.innerText||'').replace(/\\s+/g,' ').length});})()");
  console.log('AFTER LOGIN:', afterLogin.result.value);

  // ---- LIGHT snapshot ----
  const light = await snap('light');
  console.log('LIGHT:', JSON.stringify(light));

  // ---- toggle to DARK ----
  const tclk = await click('.basic-layout__theme-toggle');
  console.log('TOGGLE CLICK:', tclk.result.value);
  await sleep(1500);
  const dark = await snap('dark');
  console.log('DARK:', JSON.stringify(dark));

  // ---- toggle back to LIGHT (verify two-way) ----
  await click('.basic-layout__theme-toggle');
  await sleep(1200);
  const back = await ev("(function(){return document.documentElement.getAttribute('theme-mode');})()");
  console.log('AFTER 2nd TOGGLE themeMode:', back.result.value);

  // ---- assertions ----
  const checks = {
    themeTogglePresent: light.togglePresent === true,
    lightIsLight: (light.themeMode === 'light' || light.themeMode === null),
    darkIsDark: dark.themeMode === 'dark',
    darkPersisted: dark.localStorageTheme === 'dark',
    bgChanged: light.bodyBg !== dark.bodyBg,
    brandStable: light.brand7 === dark.brand7 && light.brand7.length > 0,
    menuDark: dark.menuTheme === 'dark',
    roundTrip: back.result.value === 'light' || back.result.value === null,
    noConsoleErrors: errors.length === 0,
  };
  console.log('\n=== CHECKS ===');
  Object.entries(checks).forEach(([k, v]) => console.log((v ? 'PASS ' : 'FAIL ') + k));
  console.log('LIGHT screenshot:', light.screenshot, '| DARK screenshot:', dark.screenshot);
  console.log('\n=== CONSOLE ERRORS (' + errors.length + ') ===');
  errors.slice(0, 30).forEach(e => console.log(' - ' + e.slice(0, 300)));

  const allPass = Object.values(checks).every(Boolean);
  console.log('\n=== THEME RESULT: ' + (allPass ? 'ALL PASS' : 'SOME FAIL') + ' ===');
  writeFileSync('theme-check-result.json', JSON.stringify({ afterLogin: JSON.parse(afterLogin.result.value), light, dark, backTheme: back.result.value, checks, errors }, null, 2));
  ws.close();
});
