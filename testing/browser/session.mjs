import { spawn } from 'node:child_process';
import { open, readFile, rm } from 'node:fs/promises';
import { setTimeout as delay } from 'node:timers/promises';

async function exitedWithin(exit, milliseconds) {
  let timer;
  try {
    return await Promise.race([exit.then(() => true), new Promise(resolve => {
      timer = setTimeout(() => resolve(false), milliseconds);
    })]);
  } finally { clearTimeout(timer); }
}

// Own only one direct child. The caller owns executable admission, paths, parent
// directories, environment, readiness schema and test-body deadlines.
export async function withFixtureSession(options, run) {
  const { command, args = [], cwd, env, readyPath, logPath, startupMs, stopMs, pollMs } = options;
  for (const value of [startupMs, stopMs, pollMs]) {
    if (!Number.isSafeInteger(value) || value <= 0 || value > 2147483647) throw new Error('fixture deadlines must be integers in 1..2147483647');
  }
  if (readyPath === logPath) throw new Error('readiness and log paths must differ');
  await rm(readyPath, { force: true });
  const log = await open(logPath, 'a');
  let child;
  let outcome;
  const failures = [];
  let result;
  let exit;
  try {
    child = spawn(command, args, { cwd, env, stdio: ['pipe', log.fd, log.fd] });
    exit = new Promise(resolve => {
      child.once('error', error => { outcome = { error }; resolve(outcome); });
      child.once('exit', (code, signal) => { outcome = { code, signal }; resolve(outcome); });
    });
    child.stdin.on('error', () => {}); // A child may close stdin before cleanup.
    const deadline = performance.now() + startupMs;
    let connection;
    for (;;) {
      if (outcome) throw outcome.error ?? new Error(`fixture exited before readiness (${outcome.code ?? outcome.signal}); see ${logPath}`);
      if (performance.now() >= deadline) throw new Error(`fixture readiness timeout; see ${logPath}`);
      try { connection = JSON.parse(await readFile(readyPath, 'utf8')); break; }
      catch (error) { if (error.code !== 'ENOENT' && !(error instanceof SyntaxError)) throw error; }
      await delay(Math.min(pollMs, Math.max(1, deadline - performance.now())));
    }
    result = await run(connection);
  } catch (error) { failures.push(error); }
  finally {
    try {
      if (child && exit && !outcome) {
        child.stdin.end('\n');
        if (!await exitedWithin(exit, stopMs)) {
          child.kill('SIGTERM');
          if (!await exitedWithin(exit, stopMs)) {
            child.kill('SIGKILL');
            if (!await exitedWithin(exit, stopMs)) throw new Error('fixture did not exit after SIGKILL');
          }
        }
      }
    } catch (error) {
      failures.push(error);
    }
    try { await log.close(); } catch (error) { failures.push(error); }
  }
  if (failures.length === 1) throw failures[0];
  if (failures.length > 1) throw new AggregateError(failures, 'fixture body and cleanup failed');
  return result;
}
