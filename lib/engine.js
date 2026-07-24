const { execSync } = require('child_process');
const os = require('os');
const fs = require('fs');
const path = require('path');

function execCmd(cmd, timeout = 10000) {
  try { return execSync(cmd, { timeout, encoding: 'utf8', stdio: ['pipe', 'pipe', 'pipe'] }); } catch { return ''; }
}

function parseHostPort(n) {
  let host, portStr;
  if (n[0] === '[') { const i = n.indexOf(']'); host = n.slice(1, i); portStr = n.slice(i + 2); }
  else { const cp = n.lastIndexOf(':'); host = n.slice(0, cp); portStr = n.slice(cp + 1); }
  return { host, port: parseInt(portStr, 10) };
}

function normalizeBind(host, family) {
  if (host === '*' || host === '') return family === 'IPv6' ? '::' : '0.0.0.0';
  return host;
}

function scanSockets() {
  const output = execCmd('lsof -nP -iTCP -iUDP -F pcftPnT');
  const all = [];
  let pid = null, cmd = '';
  let fam = null, proto = null, name = null, state = null;
  const flushFile = () => {
    if (name != null) {
      const isConn = name.indexOf('->') >= 0;
      const local = isConn ? name.split('->')[0] : name;
      const { host, port } = parseHostPort(local);
      if (!isNaN(port) && port > 0 && port <= 65535) {
        all.push({
          pid, process_name: cmd,
          protocol: proto || 'TCP', family: fam || 'IPv4',
          bind_address: normalizeBind(host, fam), port,
          state: state || (proto === 'UDP' ? 'UDP' : null),
          conn: isConn,
        });
      }
    }
    fam = proto = name = state = null;
  };
  for (const line of output.split('\n')) {
    if (!line) continue;
    const tag = line[0], val = line.slice(1);
    if (tag === 'p') { flushFile(); pid = parseInt(val, 10) || null; cmd = ''; }
    else if (tag === 'c') { cmd = val; }
    else if (tag === 'f') { flushFile(); }
    else if (tag === 't') { fam = val; }
    else if (tag === 'P') { proto = val; }
    else if (tag === 'n') { name = val; }
    else if (tag === 'T') { if (val.startsWith('ST=')) state = val.slice(3); }
  }
  flushFile();
  const listeners = all.filter(s => !s.conn && (s.protocol === 'UDP' || s.state === 'LISTEN'));
  const seen = new Set();
  const deduped = listeners.filter(s => {
    const k = `${s.port}-${s.protocol}-${s.bind_address}-${s.pid}`;
    if (seen.has(k)) return false; seen.add(k); return true;
  });
  deduped.sort((a, b) => a.port - b.port || String(a.bind_address).localeCompare(String(b.bind_address)));
  return { listeners: deduped, all };
}

function suggestFree(start, end, liveUsed, declaredUsed) {
  for (let c = start; c <= end; c++) if (!liveUsed.has(c) && !declaredUsed.has(c)) return c;
  for (let c = start; c <= end; c++) if (!liveUsed.has(c)) return c;
  return null;
}

const PROCESS_NAME_MAP = {
  'com.docke':   'Docker Desktop',
  'Docker':      'Docker Desktop',
  'docker':      'Docker',
  'ControlCe':   'AirPlay Receiver',
  'rapportd':    'AirPlay / Handoff',
  'LM Studio':   'LM Studio',
  'node':        'Node.js',
  'node.exe':    'Node.js',
  'next-server': 'Next.js',
  'next-dev':    'Next.js (dev)',
  'vite':        'Vite',
  'python3.1':   'Hermes Agent',
  'python3':     'Python',
  'python':      'Python',
  'ollama':      'Ollama',
  'Ollama':      'Ollama',
  'keybase':     'Keybase',
  'kbfs':        'Keybase FS',
  'Raycast':     'Raycast',
  'Electron':    'Perci',
  'Antigravi':   'Antigravity',
  'app_inkwe':   'Inkweasel',
  'language_':   'Language Server',
  'lmlink-co':   'LM Link',
  'Mountain':    'Mountain',
  'sshd':        'SSH',
  'postgres':    'PostgreSQL',
  'redis-server':'Redis',
  'nginx':       'nginx',
};

function friendlyProcessName(raw) {
  if (!raw) return '—';
  if (PROCESS_NAME_MAP[raw]) return PROCESS_NAME_MAP[raw];
  const base = raw.replace(/\d+(\.\d+)*$/, '').toLowerCase();
  if (base === 'python') return 'Python';
  if (base === 'node')   return 'Node.js';
  if (raw.length > 9 && PROCESS_NAME_MAP[raw.slice(0, 9)]) return PROCESS_NAME_MAP[raw.slice(0, 9)];
  return raw.replace(/_/g, ' ').replace(/\b\w/g, c => c.toUpperCase());
}

function loadPortmasters() {
  const home = os.homedir();
  const files = [];
  const check = (p) => { if (fs.existsSync(p)) files.push(p); };
  check(path.join(home, '.config/agent-rules/PORTMASTER.md'));
  const walk = (dir, maxDepth) => {
    if (maxDepth <= 0 || !fs.existsSync(dir)) return;
    try { for (const entry of fs.readdirSync(dir, { withFileTypes: true })) { const full = path.join(dir, entry.name); if (entry.isDirectory()) walk(full, maxDepth - 1); else if (entry.name === 'PORTMASTER.md') files.push(full); } } catch { /* skip */ }
  };
  for (const d of ['projects', 'code', 'dev', 'workspace', 'src']) { walk(path.join(home, d), 4); }
  try {
    for (const entry of fs.readdirSync(home, { withFileTypes: true })) {
      if (entry.isDirectory() && !entry.name.startsWith('.') && !['Library', 'Applications', 'node_modules'].includes(entry.name)) {
        walk(path.join(home, entry.name), 3);
      }
    }
  } catch { /* skip */ }
  const entries = [];
  for (const f of [...new Set(files)]) {
    try {
      const content = fs.readFileSync(f, 'utf8');
      const tableRe = /\|\s*(\d+)\s*\|\s*([^|]+?)\s*\|\s*([^|]+?)\s*\|\s*([^|]+?)\s*\|\s*([^|]+?)\s*\|\s*([^|]+?)\s*\|/;
      for (const line of content.split('\n')) {
        const m = tableRe.exec(line);
        if (m) entries.push({ port: parseInt(m[1], 10), service: m[2].trim(), protocol: m[3].trim(), bind: m[4].trim(), managed_by: m[5].trim(), notes: m[6].trim(), source_file: f });
      }
    } catch { /* skip */ }
  }
  return entries;
}

function detectConflicts(listeners, entries) {
  const conflicts = [];
  const liveUsed = new Set(listeners.map(l => l.port));
  const declaredUsed = new Set(entries.map(e => e.port));
  const norm = (s) => String(s || '').toLowerCase().replace(/[^a-z0-9]/g, '');
  const STOP = new Set(['app', 'com', 'the', 'backend', 'desktop', 'server', 'agent', 'service', 'daemon', 'run', 'serve', 'dev', 'api', 'node', 'python', 'main']);
  const tokens = (s) => String(s || '').toLowerCase().split(/[^a-z0-9]+/).filter(t => t.length >= 3 && !STOP.has(t));
  const related = (a, b) => {
    const na = norm(a), nb = norm(b);
    if (na && nb && (na.includes(nb) || nb.includes(na))) return true;
    const tb = new Set(tokens(b));
    return tokens(a).some(t => tb.has(t));
  };

  const byPort = {};
  for (const e of entries) (byPort[e.port] = byPort[e.port] || []).push(e);
  for (const portKey of Object.keys(byPort)) {
    const group = byPort[portKey];
    const owners = [...new Set(group.map(e => norm(e.managed_by) || norm(e.service)).filter(Boolean))];
    if (owners.length > 1) {
      const a = group[0];
      const b = group.find(e => (norm(e.managed_by) || norm(e.service)) !== (norm(a.managed_by) || norm(a.service))) || group[1];
      const liveDup = listeners.find(l => l.port === Number(portKey) && l.pid);
      conflicts.push({
        port: Number(portKey), kind: 'duplicate_declaration',
        process_a: a.managed_by || a.service, pid_a: null,
        process_b: b.managed_by || b.service, pid_b: null,
        suggestion: suggestFree(3000, 3999, liveUsed, declaredUsed),
        explanation: `Port ${portKey} is declared to two different owners ("${a.managed_by || a.service}" and "${b.managed_by || b.service}") across PORTMASTER.md files${liveDup ? `, and is currently held by "${friendlyProcessName(liveDup.process_name)}" (PID ${liveDup.pid})` : ''}.`,
      });
    }
  }

  for (const l of listeners) {
    const decl = entries.find(e => e.port === l.port);
    if (!decl) continue;
    const declaredOwner = decl.managed_by || decl.service;
    if (declaredOwner && !related(declaredOwner, l.process_name) && !related(declaredOwner, friendlyProcessName(l.process_name))) {
      conflicts.push({
        port: l.port, kind: 'owner_mismatch',
        process_a: l.process_name, pid_a: l.pid,
        process_b: declaredOwner, pid_b: null,
        decl_source_file: decl.source_file,
        suggestion: suggestFree(3000, 3999, liveUsed, declaredUsed),
        explanation: `Port ${l.port} is declared for "${declaredOwner}" but is actually held by "${friendlyProcessName(l.process_name)}" (PID ${l.pid}).`,
      });
    }
  }

  const procsByPort = {};
  for (const l of listeners) {
    if (!l.pid) continue;
    const arr = (procsByPort[l.port] = procsByPort[l.port] || []);
    if (!arr.some(p => p.pid === l.pid)) arr.push({ pid: l.pid, name: l.process_name, port: l.port });
  }
  const allLiveProcs = [];
  for (const l of listeners) {
    if (!l.pid) continue;
    if (!allLiveProcs.some(p => p.pid === l.pid)) allLiveProcs.push({ pid: l.pid, name: l.process_name, port: l.port });
  }
  for (const c of conflicts) {
    const procs = [...(procsByPort[c.port] || [])];
    const addSecondary = (ownerName) => {
      if (!ownerName) return;
      for (const lp of allLiveProcs) {
        if (!procs.some(p => p.pid === lp.pid) && related(ownerName, lp.name)) {
          procs.push({ pid: lp.pid, name: lp.name, port: lp.port, secondary: true });
        }
      }
    };
    addSecondary(c.process_a);
    addSecondary(c.process_b);
    c.processes = procs;
  }

  const seen = new Set();
  return conflicts.filter(c => { const k = `${c.port}-${c.kind}`; if (seen.has(k)) return false; seen.add(k); return true; });
}

function scanPorts() {
  const { listeners } = scanSockets();
  const portmasterEntries = loadPortmasters();
  const declared = new Set(portmasterEntries.map(e => e.port));
  for (const p of listeners) {
    const pm = portmasterEntries.find(e => e.port === p.port);
    if (pm) { p.service_name = pm.service; p.managed_by = pm.managed_by; }
    p.exposed = (p.bind_address === '0.0.0.0' || p.bind_address === '::');
    p.undeclared = !declared.has(p.port);
    p.source = 'Live';
  }
  const conflicts = detectConflicts(listeners, portmasterEntries);
  return { ports: listeners, conflicts, portmaster_entries: portmasterEntries };
}

function checkPort(port) {
  const { listeners, all } = scanSockets();
  const entry = listeners.find(p => p.port === port);
  const liveUsed = new Set(listeners.map(p => p.port));
  const declared = new Set(loadPortmasters().map(e => e.port));
  const suggestion = entry ? suggestFree(port + 1, 65535, liveUsed, declared) : null;
  const transient = !entry ? all.find(s => s.port === port && s.protocol === 'TCP' && s.state && s.state !== 'LISTEN') : null;
  return {
    port, in_use: !!entry,
    process: entry ? entry.process_name : (transient ? transient.process_name : null),
    pid: entry ? entry.pid : (transient ? transient.pid : null),
    bind: entry ? entry.bind_address : null,
    protocol: entry ? entry.protocol : null,
    transient_state: transient ? transient.state : null,
    suggestion,
  };
}

function suggestPort(start = 3000, end = 3999) {
  const { listeners } = scanSockets();
  const liveUsed = new Set(listeners.map(p => p.port));
  const declared = new Set(loadPortmasters().map(e => e.port));
  return suggestFree(start, end, liveUsed, declared) || start;
}

function getPortmasterFiles() {
  return [...new Set(loadPortmasters().map(p => p.source_file))];
}

module.exports = {
  execCmd, parseHostPort, normalizeBind, scanSockets, suggestFree,
  detectConflicts, loadPortmasters, friendlyProcessName, PROCESS_NAME_MAP,
  scanPorts, checkPort, suggestPort, getPortmasterFiles,
};
