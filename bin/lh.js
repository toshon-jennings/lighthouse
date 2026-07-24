#!/usr/bin/env node

const { Command } = require('commander');
const { scanPorts, checkPort, suggestPort, getPortmasterFiles, friendlyProcessName } = require('../lib/engine');
const pkg = require('../package.json');

const program = new Command();

program
  .name('lh')
  .description('Lighthouse — local development port awareness')
  .version(pkg.version);

program
  .command('list')
  .description('list live ports')
  .option('--json', 'output as JSON')
  .action((opts) => {
    const { ports, conflicts } = scanPorts();
    if (opts.json) {
      console.log(JSON.stringify({ ports, conflicts }, null, 2));
      return;
    }
    if (ports.length === 0) {
      console.log('No listening ports found.');
      return;
    }
    console.log(`PORT   BIND          PROCESS              SERVICE            SOURCE`);
    console.log(`----   ----          -------              -------            ------`);
    for (const p of ports) {
      const port = String(p.port).padEnd(6);
      const bind = (p.bind_address || '').padEnd(13);
      const proc = friendlyProcessName(p.process_name).padEnd(20);
      const svc = (p.service_name || (p.undeclared ? '(undeclared)' : '')).padEnd(18);
      const flags = [];
      if (p.exposed) flags.push('exposed');
      if (p.undeclared) flags.push('undeclared');
      const src = (p.source || 'Live') + (flags.length ? ` [${flags.join(', ')}]` : '');
      console.log(`${port}${bind}${proc}${svc}${src}`);
    }
    if (conflicts.length > 0) {
      console.log(`\n⚠ ${conflicts.length} conflict(s) detected:`);
      for (const c of conflicts) {
        console.log(`  Port ${c.port}: ${c.explanation}`);
      }
    }
  });

program
  .command('check <port>')
  .description('check if a port is free')
  .option('--json', 'output as JSON')
  .action((portStr, opts) => {
    const port = parseInt(portStr, 10);
    if (isNaN(port) || port < 1 || port > 65535) {
      console.error(`Invalid port: ${portStr}`);
      process.exit(1);
    }
    const result = checkPort(port);
    if (opts.json) {
      console.log(JSON.stringify(result, null, 2));
      return;
    }
    if (result.in_use) {
      console.log(`Port ${port} is IN USE by ${friendlyProcessName(result.process)} (PID ${result.pid}) on ${result.bind}`);
    } else if (result.transient_state) {
      console.log(`Port ${port} is not listening but has a lingering ${result.transient_state} socket (${result.process}, PID ${result.pid})`);
    } else {
      console.log(`Port ${port} is free`);
    }
    if (result.suggestion) {
      console.log(`Suggested alternative: ${result.suggestion}`);
    }
  });

program
  .command('suggest')
  .description('suggest a free port')
  .option('-r, --range <start:end>', 'port range to search (default: 3000:3999)', '3000:3999')
  .option('--json', 'output as JSON')
  .action((opts) => {
    const [startStr, endStr] = opts.range.split(':');
    const start = parseInt(startStr, 10) || 3000;
    const end = parseInt(endStr, 10) || 3999;
    const port = suggestPort(start, end);
    if (opts.json) {
      console.log(JSON.stringify({ port, range: [start, end] }, null, 2));
      return;
    }
    console.log(port);
  });

program
  .command('portmasters')
  .description('list PORTMASTER.md files')
  .option('--json', 'output as JSON')
  .action((opts) => {
    const files = getPortmasterFiles();
    if (opts.json) {
      console.log(JSON.stringify(files, null, 2));
      return;
    }
    if (files.length === 0) {
      console.log('No PORTMASTER.md files found.');
      return;
    }
    for (const f of files) {
      console.log(f);
    }
  });

program.parse();
