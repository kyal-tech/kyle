import * as vscode from 'vscode';
import * as fs from 'fs';
import * as path from 'path';
import { LanguageClient, LanguageClientOptions, ServerOptions } from 'vscode-languageclient/node';
import { KyleTaskProvider } from './tasks';
import { KyleTestController } from './testUI';

let client: LanguageClient | null = null;
let testController: KyleTestController | null = null;

function findKlBinary(): string | null {
    const config = vscode.workspace.getConfiguration('ky');
    const configured = config.get<string>('kycPath');
    if (configured && configured !== 'ky') {
        if (fs.existsSync(configured)) return configured;
    }

    const isWindows = process.platform === 'win32';
    const exeName = isWindows ? 'ky.exe' : 'ky';

    // 1. Search PATH
    const envPath = process.env.PATH || '';
    const dirs = envPath.split(path.delimiter);
    for (const dir of dirs) {
        const candidate = path.join(dir, exeName);
        if (fs.existsSync(candidate)) return candidate;
    }

    // 2. Search common install locations
    const home = process.env.HOME || process.env.USERPROFILE || '';
    const locations: string[] = [];

    if (isWindows) {
        locations.push(
            path.join(home, '.ky', 'bin', 'ky.exe'),
            path.join(process.env.LOCALAPPDATA || '', '.ky', 'bin', 'ky.exe'),
        );
    } else {
        locations.push(
            path.join(home, '.ky', 'bin', 'ky'),
            path.join(home, '.cargo', 'bin', 'ky'),
            '/usr/local/bin/ky',
            '/opt/homebrew/bin/ky',
            '/usr/bin/ky',
        );
    }
    for (const loc of locations) {
        if (fs.existsSync(loc)) return loc;
    }

    // 3. Try 'which' / 'where' command
    try {
        const cmd = isWindows ? 'where' : 'which';
        const result = require('child_process')
            .execSync(`${cmd} ${exeName}`, { encoding: 'utf8', stdio: ['pipe', 'pipe', 'ignore'] })
            .trim()
            .split('\n')[0]; // take first match on Windows
        if (result && fs.existsSync(result)) return result;
    } catch (_) {}

    return null;
}

export function activate(context: vscode.ExtensionContext) {
    console.log('KL Language Support activating...');

    // Register test controller (Testing UI)
    testController = new KyleTestController();
    context.subscriptions.push({ dispose: () => testController?.dispose() });

    // Register task provider
    context.subscriptions.push(
        vscode.tasks.registerTaskProvider('ky', new KyleTaskProvider())
    );

    // Register commands
    context.subscriptions.push(
        vscode.commands.registerCommand('ky.run', () => runFile('run')),
        vscode.commands.registerCommand('ky.build', () => runFile('build')),
        vscode.commands.registerCommand('ky.check', () => runFile('check')),
        vscode.commands.registerCommand('ky.test', () => runFile('test')),
        vscode.commands.registerCommand('ky.runTest', (fileUri: string, testName: string) => {
            runSpecificTest(fileUri, testName);
        })
    );

    // Register diagnostics collection
    const diagnosticCollection = vscode.languages.createDiagnosticCollection('ky');
    context.subscriptions.push(diagnosticCollection);

    // Parse output for diagnostics
    context.subscriptions.push(
        vscode.commands.registerCommand('ky.handleOutput', (output: string) => {
            parseAndSetDiagnostics(output, diagnosticCollection);
        })
    );

    // Start LSP client
    let klPath = findKlBinary();
    if (!klPath) {
        klPath = 'ky';
    }
    if (fs.existsSync(klPath) || klPath === 'ky') {
        try {
            startLanguageClient(context, klPath);
            console.log('KY language server started:', klPath);
        } catch (err) {
            console.error('Failed to start KL language server:', err);
            vscode.window.showWarningMessage(
                'KY language server could not start. Check that kl is installed and try again.'
            );
        }
    } else {
        console.warn('kl binary not found in PATH or common install locations');
        vscode.window.showWarningMessage(
            'KY language server not available. Install kl or set "ky.kycPath" in settings.'
        );
    }
}

function startLanguageClient(context: vscode.ExtensionContext, klPath: string) {
    const serverOptions: ServerOptions = {
        command: klPath,
        args: ['lsp'],
    };

    const clientOptions: LanguageClientOptions = {
        documentSelector: [{ scheme: 'file', language: 'kl' }, { scheme: 'file', language: 'kyx' }],
        synchronize: {
            fileEvents: vscode.workspace.createFileSystemWatcher('**/{*.ky,ky.toml}'),
        },
    };

    const lspClient = new LanguageClient(
        'klLanguageServer',
        'KL Language Server',
        serverOptions,
        clientOptions
    );

    client = lspClient;
    lspClient.start();
    context.subscriptions.push({ dispose: () => lspClient.stop() });
}

async function runFile(subcommand: string) {
    const editor = vscode.window.activeTextEditor;
    if (!editor) {
        vscode.window.showErrorMessage('No active editor');
        return;
    }

    const filePath = editor.document.uri.fsPath;
    if (!filePath.endsWith('.ky')) {
        vscode.window.showErrorMessage('Not a KL file');
        return;
    }

    let klPath = findKlBinary();
    if (!klPath) {
        klPath = 'ky';
    }

    const terminal = vscode.window.createTerminal('KL');
    terminal.show();
    terminal.sendText(`${klPath} ${subcommand} "${filePath}"`);
}

function runSpecificTest(fileUri: string, testName: string) {
    const filePath = fileUri.replace(/^file:\/\//, '');
    let klPath = findKlBinary();
    if (!klPath) {
        klPath = 'ky';
    }

    // Create wrapper to run just this test
    const fs = require('fs');
    const path = require('path');
    const source = fs.readFileSync(filePath, 'utf-8');
    const dir = path.dirname(filePath);
    const ext = path.extname(filePath);

    const tempDir = path.join(dir, '.ky-test');
    if (!fs.existsSync(tempDir)) {
        fs.mkdirSync(tempDir, { recursive: true });
    }

    const wrapperSource = source + `\nfn main() i32:\n    ${testName}()\n    0\n`;
    const wrapperFile = path.join(tempDir, `run_${testName}${ext}`);
    fs.writeFileSync(wrapperFile, wrapperSource);

    const terminal = vscode.window.createTerminal(`KL Test: ${testName}`);
    terminal.show();
    terminal.sendText(`${klPath} run "${wrapperFile}"`);
}

function parseAndSetDiagnostics(output: string, collection: vscode.DiagnosticCollection) {
    collection.clear();

    const diagnosticRegex = /^(.+?):(\d+):(\d+):\s*(error|warning)\[([^\]]+)\]:\s*(.+)$/gm;
    let match: RegExpExecArray | null;

    const diagnosticsByFile = new Map<string, vscode.Diagnostic[]>();

    while ((match = diagnosticRegex.exec(output)) !== null) {
        const [, file, lineStr, colStr, severity, code, message] = match;
        const line = parseInt(lineStr) - 1;
        const col = parseInt(colStr) - 1;
        const range = new vscode.Range(line, col, line, col + 1);
        const diagnostic = new vscode.Diagnostic(
            range,
            `[${code}] ${message}`,
            severity === 'error' ? vscode.DiagnosticSeverity.Error : vscode.DiagnosticSeverity.Warning
        );
        diagnostic.code = code;
        diagnostic.source = 'ky';

        const filePath = path.resolve(vscode.workspace.rootPath || '', file);
        const existing = diagnosticsByFile.get(filePath) || [];
        existing.push(diagnostic);
        diagnosticsByFile.set(filePath, existing);
    }

    for (const [file, diags] of diagnosticsByFile) {
        collection.set(vscode.Uri.file(file), diags);
    }
}

export function deactivate() {
    if (testController) {
        testController.dispose();
        testController = null;
    }
    if (client) {
        return client.stop();
    }
    return undefined;
}
