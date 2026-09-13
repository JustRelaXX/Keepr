// Real WebKitGTK + Tauri + SQLite end-to-end tests. No IPC or persistence mocks.
// Run after `npm run tauri build -- --debug --no-bundle` on Linux.
import { spawn } from 'node:child_process';
import { mkdtemp, mkdir, writeFile, rm, readFile, readdir } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { resolve, join } from 'node:path';
import assert from 'node:assert/strict';

const root = resolve(import.meta.dirname, '..');
const binary = resolve(root, process.env.KEEPR_BINARY ?? 'target/debug/keepr');
const profile = await mkdtemp(join(tmpdir(), 'keepr-e2e-'));
const output = join(root, 'test-results');
await mkdir(output, { recursive: true });
const display = ':97';
const env = { ...process.env, HOME:profile, DISPLAY: display, GDK_BACKEND: 'x11', XDG_DATA_HOME: join(profile,'data'), XDG_CONFIG_HOME: join(profile,'config'), XDG_CACHE_HOME: join(profile,'cache'), WEBKIT_DISABLE_COMPOSITING_MODE:'1', TAURI_WEBVIEW_AUTOMATION:'true' };
const xvfb = spawn('Xvfb', [display, '-screen', '0', '1400x1000x24', '-ac'], {env, stdio:'ignore'});
const driver = spawn('WebKitWebDriver', ['--port=4445'], {env, stdio:['ignore','pipe','pipe']});
let driverLog = '';
driver.stdout.on('data',data=>driverLog+=data);
driver.stderr.on('data',data=>driverLog+=data);
let session;
let monitor;
let notifications = '';
const base='http://127.0.0.1:4445';
const wait = ms=>new Promise(r=>setTimeout(r,ms));

async function request(method,path,body){
  const result=await fetch(`${base}${path}`,{method,headers:{'Content-Type':'application/json'},body:body===undefined?undefined:JSON.stringify(body),signal:AbortSignal.timeout(45000)});
  const data=await result.json();
  if(data.value?.error)throw new Error(`${data.value.error}: ${data.value.message}`);
  return data.value;
}
async function execute(script,args=[]){return request('POST',`/session/${session}/execute/sync`,{script,args});}
async function until(check,description){
  let last;
  for(let i=0;i<120;i++){try{if(await check())return;}catch(e){last=e;}await wait(100);}
  throw new Error(`Timed out: ${description}${last?` (${last.message})`:''}`);
}
async function has(selector){return execute('return !!document.querySelector(arguments[0])',[selector]);}
async function click(selector){
  await until(()=>has(selector),selector);
  const element=await request('POST',`/session/${session}/element`,{using:'css selector',value:selector});
  try {
    await request('POST',`/session/${session}/element/${element['element-6066-11e4-a52e-4f735466cecf']}/click`,{});
  } catch(e) {
    // Some WebKitGTK builds omit native pointer synthesis. DOM activation still
    // runs the production Svelte handlers and real Rust IPC/database path.
    if(!e.message.includes('unsupported operation'))throw e;
    await execute('const el=document.querySelector(arguments[0]);if(el.disabled)throw new Error("disabled control");el.click();',[selector]);
  }
}
async function fill(selector,value){
  await execute(`const el=document.querySelector(arguments[0]); el.value=arguments[1];el.dispatchEvent(new Event('input',{bubbles:true}));el.dispatchEvent(new Event('change',{bubbles:true}));`,[selector,value]);
}
async function clickText(selector,text){
  await until(()=>execute(`return [...document.querySelectorAll(arguments[0])].some(el=>el.textContent.trim()===arguments[1]);`,[selector,text]),text);
  await execute(`const el=[...document.querySelectorAll(arguments[0])].find(el=>el.textContent.trim()===arguments[1]);el.click();`,[selector,text]);
}
async function screenshot(name){
  if(name!=='failure')await execute('document.querySelector(".toast .icon-button")?.click();');
  await wait(300);
  const png=await request('GET',`/session/${session}/screenshot`);
  await writeFile(join(output,`${name}.png`),Buffer.from(png,'base64'));
}
async function start(){
  const data=await request('POST','/session',{capabilities:{alwaysMatch:{'webkitgtk:browserOptions':{binary,args:[]}}}});
  session=data.sessionId;
  await until(()=>has('.welcome, .app-shell'),'app startup');
}
async function stop(){if(session){await request('DELETE',`/session/${session}`);session=null;await wait(600);}}
async function closeDialog(){await click('dialog[open] .modal-header .icon-button');await until(async()=>!await has('dialog[open]'),'dialog closes');}
async function dismissTour(){for(let i=0;i<50&&!await has('.tour-tooltip');i++)await wait(100);if(await has('.tour-tooltip'))await click('[data-testid=tour-skip]');await until(async()=>!await has('.tour-tooltip'),'guide dismissed');}

try {
  await until(async()=>{try{await request('GET','/status');return true;}catch{return false;}},'WebKitWebDriver');
  await start();
  await screenshot('onboarding');
  if(process.argv.includes('--demo')) {
    await click('[data-testid=start-demo]');
    await until(()=>has('.health-panel'),'demo home');
    await dismissTour();
    await click('.settings-nav');
    await until(()=>has('.settings-layout'),'settings');
    await execute(`const select=document.querySelectorAll('.settings-layout select')[1];select.value='light';select.dispatchEvent(new Event('change',{bubbles:true}));`);
    await click('.settings-save button');
    await until(()=>execute('return document.documentElement.dataset.theme==="light";'),'light theme');
    await click('.main-nav button:nth-child(1)');
    await screenshot('home-light');
    await click('.main-nav button:nth-child(2)');
    await screenshot('things-light');
    await click('.item-main');
    await screenshot('thing-details');
    await closeDialog();
    await click('[data-testid=add-item]');
    await screenshot('presets');
    await closeDialog();
    await click('.main-nav button:nth-child(3)');
    await screenshot('calendar-light');
    console.log('✓ real demo home and light-theme screenshots');
  } else {
  await click('[data-testid=start-empty]');
  await click('.create-custom');
  await fill('[data-testid=item-name]','E2E water filter');
  await click('[data-testid=save-item]');
  await until(()=>has('.detail-hero'),'saved thing');
  assert.match(await execute('return document.querySelector(".detail-hero").textContent;'),/E2E water filter/);
  await closeDialog();
  console.log('✓ create recurring item through real IPC');

  await until(()=>has('.tour-tooltip'),'first-run guide appears');
  for(let i=0;i<4;i++){await click('[data-testid=tour-next]');await wait(250);}
  assert.match(await execute('return document.querySelector(".tour-tooltip h2").textContent;'),/.+/);
  await click('[data-testid=tour-done]');
  await until(async()=>!await has('.tour-tooltip'),'guide finished');
  assert.equal(await execute('return localStorage.getItem("keepr.tour.v1");'),'done');
  console.log('✓ first-run guide walkthrough');

  await click('.main-nav button:nth-child(2)');
  await until(()=>has('.item-card'),'item list');
  await click('.care-button');
  await until(()=>execute('return !!document.querySelector(".toast")?.textContent.includes("Отличная забота");'),'complete');
  await click('.item-main');
  assert.match(await execute('return document.querySelector(".mini-timeline").textContent;'),/Забота выполнена/);
  await closeDialog();
  console.log('✓ completion records history and starts next cycle');

  await click('.item-card .popover-wrap button');
  await clickText('.action-popover button','Завтра');
  await until(()=>has('.snoozed'),'postponed');
  console.log('✓ snooze persisted');

  await click('.sidebar-section-label button');
  await fill('[data-testid=room-name]','E2E room');
  await click('dialog form button[type=submit]');
  await until(async()=>!await has('dialog[open]'),'saved room');
  await click('.item-main');
  await clickText('dialog .form-footer button','Изменить');
  await until(()=>has('[data-testid=item-name]'),'editing');
  await execute(`const select=document.querySelector('dialog select');const option=[...select.options].find(o=>o.textContent==='E2E room');select.value=option.value;select.dispatchEvent(new Event('change',{bubbles:true}));`);
  await click('[data-testid=save-item]');
  await until(()=>has('.detail-hero'),'updated item');
  assert.match(await execute('return document.querySelector(".detail-hero").textContent;'),/E2E room/);
  await closeDialog();
  console.log('✓ create room and move item');

  await clickText('.main-nav button','Календарь');
  await until(()=>has('.calendar-grid'),'calendar');
  assert.equal(await execute('return document.querySelectorAll(".calendar-day").length;'),42);
  await screenshot('calendar');
  await clickText('.main-nav button','Журнал');
  await until(()=>has('.journal-entry'),'journal');
  assert.ok(await execute('return document.querySelectorAll(".journal-entry").length>=4;'));
  console.log('✓ real calendar and journal');

  const journalCount = await execute('return document.querySelectorAll(".journal-entry").length;');
  const removedEventId = await execute('return document.querySelector(".journal-event-icon[data-kind=done]").closest(".journal-entry").dataset.eventId;');
  const removedRow = `.journal-entry[data-event-id="${removedEventId}"]`;
  await screenshot('journal-actions');
  await click(`${removedRow} .journal-delete`);
  await until(async()=>!await has(removedRow),'journal entry removed');
  assert.equal(await execute('return document.querySelectorAll(".journal-entry").length;'),journalCount-1);
  assert.ok(!await has('dialog[open]'),'deleting a message does not open its item');
  assert.ok(await execute('return document.activeElement.matches(".journal-delete");'),'keyboard focus stays on a journal action');
  await clickText('.toast button','Вернуть');
  await until(()=>has(removedRow),'journal deletion undone');
  assert.equal(await execute('return document.querySelectorAll(".journal-entry").length;'),journalCount);
  await click(`${removedRow} .journal-delete`);
  await until(async()=>!await has(removedRow),'journal entry removed again');
  console.log('✓ delete individual journal entry and undo without changing the item');

  await stop();
  await start();
  await click('.main-nav button:nth-child(4)');
  await until(()=>has('.journal-entry'),'journal after restart');
  assert.ok(!await has(removedRow));
  assert.equal(await execute('return document.querySelectorAll(".journal-entry").length;'),journalCount-1);
  console.log('✓ journal deletion survives a full process restart');
  await click('.main-nav button:nth-child(2)');
  await until(()=>has('.item-card'),'restored item');
  assert.match(await execute('return document.querySelector(".item-card").textContent;'),/E2E water filter/);
  assert.ok(await has('.snoozed'));
  console.log('✓ full process restart preserves SQLite data');

  await click('.item-main');
  await click('[data-testid=delete-item]');
  await until(()=>has('.empty-state'),'removed item');
  await clickText('.toast button','Вернуть');
  await until(()=>has('.item-card'),'undo deletion');
  console.log('✓ delete and undo');

  await click('[data-testid=add-item]');
  await click('.create-custom');
  await fill('[data-testid=item-name]','E2E one-time');
  await fill('[data-testid=repeat]','once');
  await until(()=>execute('return !document.querySelector("[data-testid=save-item]").disabled;'),'date calculation');
  await click('[data-testid=save-item]');
  await until(()=>has('.detail-actions'),'one-time saved');
  await click('.detail-actions .primary');
  await until(()=>has('dialog .editor-form'),'one-time completed');
  await click('dialog .editor-form .primary');
  await until(()=>has('.detail-actions'),'new cycle');
  await closeDialog();
  console.log('✓ one-time completion and new event with preserved history');

  await click('.settings-nav');
  await until(()=>has('.settings-layout'),'settings');
  await click('.settings-main > .settings-section:nth-of-type(3) input');
  const startupDir=join(profile,'config/autostart');
  await until(async()=>{try{return (await readdir(startupDir)).some(f=>f.endsWith('.desktop'));}catch{return false;}},'autostart registration');
  const startupFile=(await readdir(startupDir)).find(f=>f.endsWith('.desktop'));
  assert.match(await readFile(join(startupDir,startupFile),'utf8'),/--background/);
  await click('.settings-main > .settings-section:nth-of-type(3) input');
  await until(async()=>!(await readdir(startupDir)).includes(startupFile),'autostart removal');
  console.log('✓ autostart enable/disable in isolated home directory');
  if(process.env.KEEPR_TEST_NOTIFICATIONS==='1') {
    monitor=spawn('dbus-monitor',['--session',"type='method_call',interface='org.freedesktop.Notifications',member='Notify'"],{env,stdio:['ignore','pipe','ignore']});
    monitor.stdout.on('data',data=>notifications+=data);
    await wait(300);
    await clickText('button','Проверить уведомление');
    await until(()=>Promise.resolve(notifications.includes('Keepr')),'native notification delivered over D-Bus');
    console.log('✓ native system notification (no buttons)');
    notifications='';
    await execute(`for(const el of document.querySelectorAll('.settings-layout input[type=time]')){el.value='00:00';el.dispatchEvent(new Event('input',{bubbles:true}));el.dispatchEvent(new Event('change',{bubbles:true}));}`);
    await click('.settings-save button');
    await until(()=>Promise.resolve(notifications.includes('E2E one-time')),'scheduler triggers real due notification');
    console.log('✓ scheduler wakes on settings change and delivers a due reminder');
  }
  await execute(`const selects=[...document.querySelectorAll('.settings-layout select')];selects[0].value='en';selects[0].dispatchEvent(new Event('change',{bubbles:true}));selects[1].value='dark';selects[1].dispatchEvent(new Event('change',{bubbles:true}));`);
  await click('.settings-save button');
  await until(()=>execute('return document.documentElement.lang==="en"&&document.documentElement.dataset.theme==="dark";'),'theme and language');
  await screenshot('settings-dark');
  await clickText('.main-nav button','My home');
  await screenshot('dashboard-dark');
  console.log('✓ dark theme and English localization');

  await click('.settings-nav');
  await until(()=>has('.settings-layout'),'settings again');
  await click('[data-testid=replay-tour]');
  await until(()=>has('.tour-tooltip'),'guide replayed from settings');
  await click('[data-testid=tour-skip]');
  await until(async()=>!await has('.tour-tooltip'),'replay dismissed');
  console.log('✓ guide replay from settings');
  }
  await stop();
  const resident=spawn(binary,['--background'],{env,stdio:'ignore'});
  try {
    await wait(2000);
    assert.equal(resident.exitCode,null,'background process stays alive');
    const status=await readFile(`/proc/${resident.pid}/status`,'utf8');
    const children=await readFile(`/proc/${resident.pid}/task/${resident.pid}/children`,'utf8');
    const names=await Promise.all(children.trim().split(/\s+/).filter(Boolean).map(pid=>readFile(`/proc/${pid}/comm`,'utf8').catch(()=>'')));
    assert.ok(!names.some(name=>name.includes('WebKit')),'background starts without a WebView');
    const memory=await readFile(`/proc/${resident.pid}/smaps_rollup`,'utf8');
    console.log(`✓ background without WebView; ${status.match(/VmRSS:\s+(\d+) kB/)?.[1]??'?'} KiB RSS, ${memory.match(/\nPss:\s+(\d+) kB/)?.[1]??'?'} KiB PSS`);
    const second=spawn(binary,['--background'],{env,stdio:'ignore'});
    await new Promise((resolve,reject)=>{second.once('exit',resolve);setTimeout(()=>{if(second.exitCode===null){second.kill();reject(new Error('duplicate process did not exit'));}},5000).unref();});
    assert.equal(resident.exitCode,null);
    console.log('✓ duplicate launch forwards to resident instance');
  } finally {resident.kill();await wait(300);}
  console.log('All desktop E2E scenarios passed. Screenshots: test-results/');
} catch(error) {
  if(session){try{await screenshot('failure');await writeFile(join(output,'failure.html'),await execute('return document.documentElement.outerHTML;'));}catch{}}
  console.error(error);process.exitCode=1;
} finally {
  try{await stop();}catch{}
  driver.kill();xvfb.kill();
  monitor?.kill();
  await writeFile(join(output,'webdriver.log'),driverLog);
  if(notifications)await writeFile(join(output,'notifications.log'),notifications);
  await rm(profile,{recursive:true,force:true});
}
