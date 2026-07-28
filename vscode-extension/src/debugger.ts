import { EventEmitter } from 'events';

interface DAPRequest {
  seq: number;
  type: 'request';
  command: string;
  arguments?: any;
}

interface DAPResponse {
  seq: number;
  type: 'response';
  request_seq: number;
  command: string;
  success: boolean;
  body?: any;
  message?: string;
}

interface DAPEvent {
  seq: number;
  type: 'event';
  event: string;
  body?: any;
}

export class KyleDebugSession extends EventEmitter {
  private seq = 1;
  private isRunning = false;
  private breakpoints = new Map<string, number[]>();
  private childProcess: any = null;

  constructor() {
    super();
  }

  start(): void {
    const buffer: string[] = [];
    process.stdin.setEncoding('utf8');
    process.stdin.on('data', (chunk: string) => {
      buffer.push(chunk);
      const content = buffer.join('');
      const parts = content.split('\r\n\r\n');
      while (parts.length >= 2) {
        const headers = parts.shift()!;
        const bodyStr = parts.join('\r\n\r\n');
        const match = headers.match(/Content-Length:\s*(\d+)/i);
        if (match) {
          const length = parseInt(match[1], 10);
          if (bodyStr.length >= length) {
            const raw = bodyStr.substring(0, length);
            buffer.length = 0;
            buffer.push(bodyStr.substring(length));
            try {
              const msg = JSON.parse(raw);
              this.handleMessage(msg);
            } catch { }
            break;
          }
        }
        buffer.length = 0;
        buffer.push(bodyStr);
        break;
      }
    });
  }

  private send(body: DAPResponse | DAPEvent): void {
    const json = JSON.stringify(body);
    const header = `Content-Length: ${Buffer.byteLength(json, 'utf8')}\r\n\r\n`;
    process.stdout.write(header + json);
  }

  private handleMessage(msg: DAPRequest): void {
    const cmd = msg.command;
    const args = msg.arguments || {};

    switch (cmd) {
      case 'initialize':
        this.send({
          seq: this.seq++, type: 'response', request_seq: msg.seq,
          command: cmd, success: true,
          body: {
            supportsConfigurationDoneRequest: true,
            supportsSetVariable: false,
            supportsExceptionInfoRequest: false,
            supportsTerminateRequest: true,
            supportTerminateDebuggee: true,
          },
        });
        break;

      case 'launch':
        this.doLaunch(args);
        this.send({
          seq: this.seq++, type: 'response', request_seq: msg.seq,
          command: cmd, success: true,
        });
        break;

      case 'setBreakpoints': {
        const file = args.source?.path || '';
        const lines = (args.breakpoints || []).map((bp: any) => bp.line);
        if (file) this.breakpoints.set(file, lines);
        const bps = lines.map((line: number) => ({
          verified: true,
          line,
          id: this.seq++,
        }));
        this.send({
          seq: this.seq++, type: 'response', request_seq: msg.seq,
          command: cmd, success: true,
          body: { breakpoints: bps },
        });
        break;
      }

      case 'setFunctionBreakpoints':
        this.send({
          seq: this.seq++, type: 'response', request_seq: msg.seq,
          command: cmd, success: true,
          body: { breakpoints: [] },
        });
        break;

      case 'setExceptionBreakpoints':
        this.send({
          seq: this.seq++, type: 'response', request_seq: msg.seq,
          command: cmd, success: true,
        });
        break;

      case 'configurationDone':
        this.runProgram();
        this.send({
          seq: this.seq++, type: 'response', request_seq: msg.seq,
          command: cmd, success: true,
        });
        break;

      case 'continue':
        this.send({
          seq: this.seq++, type: 'response', request_seq: msg.seq,
          command: cmd, success: true,
          body: { allThreadsContinued: true },
        });
        break;

      case 'next':
        this.send({
          seq: this.seq++, type: 'response', request_seq: msg.seq,
          command: cmd, success: true,
        });
        break;

      case 'stepIn':
        this.send({
          seq: this.seq++, type: 'response', request_seq: msg.seq,
          command: cmd, success: true,
        });
        break;

      case 'stepOut':
        this.send({
          seq: this.seq++, type: 'response', request_seq: msg.seq,
          command: cmd, success: true,
        });
        break;

      case 'stackTrace':
        this.send({
          seq: this.seq++, type: 'response', request_seq: msg.seq,
          command: cmd, success: true,
          body: {
            stackFrames: [
              {
                id: 1,
                name: 'main',
                line: args.startFrame || 1,
                column: 1,
                source: args.source || undefined,
              },
            ],
            totalFrames: 1,
          },
        });
        break;

      case 'scopes':
        this.send({
          seq: this.seq++, type: 'response', request_seq: msg.seq,
          command: cmd, success: true,
          body: { scopes: [] },
        });
        break;

      case 'variables':
        this.send({
          seq: this.seq++, type: 'response', request_seq: msg.seq,
          command: cmd, success: true,
          body: { variables: [] },
        });
        break;

      case 'threads':
        this.send({
          seq: this.seq++, type: 'response', request_seq: msg.seq,
          command: cmd, success: true,
          body: { threads: [{ id: 1, name: 'main' }] },
        });
        break;

      case 'evaluate':
        this.send({
          seq: this.seq++, type: 'response', request_seq: msg.seq,
          command: cmd, success: false,
          message: 'Evaluation not supported',
        });
        break;

      case 'terminate':
      case 'disconnect':
        this.cleanup();
        this.send({
          seq: this.seq++, type: 'response', request_seq: msg.seq,
          command: cmd, success: true,
        });
        this.send({ seq: this.seq++, type: 'event', event: 'terminated' });
        break;

      default:
        this.send({
          seq: this.seq++, type: 'response', request_seq: msg.seq,
          command: cmd, success: false,
          message: `Unknown command: ${cmd}`,
        });
    }
  }

  private doLaunch(args: any): void {
    const program = args.program || '';
    const klcPath = args.kycPath || 'ky';
    const { execSync } = require('child_process');
    try {
      execSync(`${klcPath} build "${program}"`, { stdio: 'pipe' });
      this.send({ seq: this.seq++, type: 'event', event: 'output', body: { output: `Compiled: ${program}\n` } });
    } catch (e: any) {
      this.send({ seq: this.seq++, type: 'event', event: 'output', body: { output: `Compile error: ${e.message}\n`, category: 'stderr' } });
    }
  }

  private runProgram(): void {
    const { spawn } = require('child_process');
    const binary = './a.out';
    this.childProcess = spawn(binary, [], { stdio: ['pipe', 'pipe', 'pipe'] });

    this.childProcess.stdout.on('data', (data: Buffer) => {
      this.send({ seq: this.seq++, type: 'event', event: 'output', body: { output: data.toString(), category: 'stdout' } });
    });

    this.childProcess.stderr.on('data', (data: Buffer) => {
      this.send({ seq: this.seq++, type: 'event', event: 'output', body: { output: data.toString(), category: 'stderr' } });
    });

    this.childProcess.on('exit', () => {
      this.send({ seq: this.seq++, type: 'event', event: 'exited', body: { exitCode: 0 } });
      this.send({ seq: this.seq++, type: 'event', event: 'terminated' });
    });
  }

  private cleanup(): void {
    if (this.childProcess) {
      this.childProcess.kill();
      this.childProcess = null;
    }
  }
}

if (require.main === module) {
  const session = new KyleDebugSession();
  session.start();
}
