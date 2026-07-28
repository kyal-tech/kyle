import * as vscode from 'vscode';
import * as fs from 'fs';
import * as path from 'path';
import { execSync, spawn } from 'child_process';

function findKlBinary(): string {
    const config = vscode.workspace.getConfiguration('ky');
    const configured = config.get<string>('kycPath');
    if (configured && configured !== 'ky') {
        if (fs.existsSync(configured)) return configured;
    }
    return 'ky';
}

const testFnRegex = /#\[test\]\s*\n\s*fn\s+([a-zA-Z_][a-zA-Z0-9_]*)\s*\(/g;

export class KyleTestController {
    private controller: vscode.TestController;
    private watchDisposable: vscode.Disposable | null = null;
    private fileTestMap = new Map<string, Set<string>>();

    constructor() {
        this.controller = vscode.tests.createTestController('klTests', 'KL Tests');
        this.controller.resolveHandler = (item) => {
            if (!item) {
                this.discoverAllTests();
            }
        };

        this.controller.createRunProfile(
            'Run Tests',
            vscode.TestRunProfileKind.Run,
            (request, token) => this.runHandler(request, token),
            true
        );

        this.controller.createRunProfile(
            'Debug Test',
            vscode.TestRunProfileKind.Debug,
            (request, token) => this.runHandler(request, token),
            false
        );

        this.watchFiles();
    }

    private watchFiles() {
        if (this.watchDisposable) {
            this.watchDisposable.dispose();
        }

        const watcher = vscode.workspace.createFileSystemWatcher('**/*.ky');

        watcher.onDidCreate((uri) => this.onFileChange(uri));
        watcher.onDidChange((uri) => this.onFileChange(uri));
        watcher.onDidDelete((uri) => this.onFileDelete(uri));

        this.watchDisposable = watcher;

        this.discoverAllTests();
    }

    private async onFileChange(uri: vscode.Uri) {
        if (!uri.fsPath.endsWith('.ky')) return;
        this.discoverTestsInFile(uri);
    }

    private async onFileDelete(uri: vscode.Uri) {
        if (!uri.fsPath.endsWith('.ky')) return;
        this.fileTestMap.delete(uri.fsPath);
        this.controller.items.delete(uri.toString());
    }

    private async discoverAllTests() {
        const workspaceFolders = vscode.workspace.workspaceFolders;
        if (!workspaceFolders) return;

        const klFiles = await vscode.workspace.findFiles('**/*.ky', '**/node_modules/**');
        for (const file of klFiles) {
            await this.discoverTestsInFile(file);
        }
    }

    private async discoverTestsInFile(uri: vscode.Uri) {
        try {
            const content = fs.readFileSync(uri.fsPath, 'utf-8');
            const testNames = this.findTestFunctions(content);

            const existing = this.fileTestMap.get(uri.fsPath) || new Set();
            const testIds = new Set<string>();

            let fileItem = this.controller.items.get(uri.toString());
            if (testNames.size === 0) {
                this.fileTestMap.delete(uri.fsPath);
                if (fileItem) {
                    this.controller.items.delete(uri.toString());
                }
                return;
            }

            if (!fileItem) {
                fileItem = this.controller.createTestItem(
                    uri.toString(),
                    path.basename(uri.fsPath),
                    uri
                );
                fileItem.canResolveChildren = true;
                this.controller.items.add(fileItem);
            }

            const children = new Map<string, vscode.TestItem>();

            for (const testName of testNames) {
                const testId = `${uri.toString()}::${testName}`;
                testIds.add(testId);
                const testItem = this.controller.createTestItem(testId, testName, uri);
                testItem.tags = [new vscode.TestTag('kl:test')];
                children.set(testId, testItem);
            }

            fileItem.children.replace([...children.values()]);

            const updated = new Set(testNames);
            this.fileTestMap.set(uri.fsPath, updated);
        } catch (err) {
            console.error(`Failed to discover tests in ${uri.fsPath}:`, err);
        }
    }

    private findTestFunctions(content: string): Set<string> {
        const names = new Set<string>();
        let match: RegExpExecArray | null;
        const re = new RegExp(testFnRegex);
        while ((match = re.exec(content)) !== null) {
            names.add(match[1]);
        }
        return names;
    }

    private async runHandler(
        request: vscode.TestRunRequest,
        token: vscode.CancellationToken
    ) {
        const run = this.controller.createTestRun(request);
        const queue: vscode.TestItem[] = [];

        if (request.include) {
            request.include.forEach((item) => queue.push(item));
        } else {
            this.controller.items.forEach((item) => queue.push(item));
        }

        for (const item of queue) {
            if (token.isCancellationRequested) {
                break;
            }

            if (item.children.size > 0) {
                item.children.forEach((child) => queue.push(child));
                continue;
            }

            await this.runSingleTest(item, run, token);
        }

        run.end();
    }

    private async runSingleTest(
        item: vscode.TestItem,
        run: vscode.TestRun,
        token: vscode.CancellationToken
    ) {
        run.started(item);

        const uri = item.uri;
        if (!uri) {
            run.failed(item, new vscode.TestMessage('No file URI'));
            return;
        }

        const filePath = uri.fsPath;
        if (!fs.existsSync(filePath)) {
            run.failed(item, new vscode.TestMessage(`File not found: ${filePath}`));
            return;
        }

        const testName = item.label;
        const source = fs.readFileSync(filePath, 'utf-8');

        const tempDir = path.join(path.dirname(filePath), '.ky-test');
        if (!fs.existsSync(tempDir)) {
            fs.mkdirSync(tempDir, { recursive: true });
        }

        const sanitizedName = testName.replace(/[^a-zA-Z0-9_]/g, '_');
        const wrapperSource = `${source}\nfn main() i32:\n    ${testName}()\n    println("PASS")\n    0\n`;
        const wrapperFile = path.join(tempDir, `${sanitizedName}_test.ky`);
        fs.writeFileSync(wrapperFile, wrapperSource);

        const outputFile = path.join(tempDir, sanitizedName);
        const klPath = findKlBinary();

        try {
            const buildResult = execSync(
                `"${klPath}" build "${wrapperFile}" -o "${outputFile}"`,
                { encoding: 'utf-8', timeout: 30000, stdio: ['pipe', 'pipe', 'pipe'] }
            );
            console.log('Build output:', buildResult);
        } catch (buildErr: any) {
            const msg = buildErr.stderr || buildErr.stdout || buildErr.message || 'Build failed';
            run.failed(item, new vscode.TestMessage(`Build error:\n${msg}`));
            this.cleanup(tempDir, sanitizedName, wrapperFile);
            return;
        }

        if (token.isCancellationRequested) {
            run.skipped(item);
            this.cleanup(tempDir, sanitizedName, wrapperFile);
            return;
        }

        return new Promise<void>((resolve) => {
            const runProc = spawn(outputFile, [], {
                stdio: ['pipe', 'pipe', 'pipe'],
                timeout: 30000,
            });

            let stdout = '';
            let stderr = '';

            runProc.stdout.on('data', (data: Buffer) => {
                stdout += data.toString();
            });

            runProc.stderr.on('data', (data: Buffer) => {
                stderr += data.toString();
            });

            runProc.on('close', (code) => {
                if (code === 0) {
                    run.passed(item, stdout ? undefined : undefined);
                } else if (stderr.includes('KL ASSERT FAILED') || stderr.includes('KL PANIC')) {
                    const msg = stderr.replace('KL ASSERT FAILED: ', '').replace('KL PANIC: ', '').trim();
                    run.failed(item, new vscode.TestMessage(msg || `Test failed with exit code ${code}`));
                } else if (stderr) {
                    run.failed(item, new vscode.TestMessage(stderr.trim()));
                } else {
                    run.failed(item, new vscode.TestMessage(`Test failed with exit code ${code}`));
                }

                if (stdout.trim()) {
                    run.appendOutput(`${stdout}\r\n`);
                }

                this.cleanup(tempDir, sanitizedName, wrapperFile);
                resolve();
            });

            runProc.on('error', (err) => {
                run.errored(item, new vscode.TestMessage(err.message));
                this.cleanup(tempDir, sanitizedName, wrapperFile);
                resolve();
            });
        });
    }

    private cleanup(tempDir: string, name: string, wrapperFile: string) {
        try {
            if (fs.existsSync(wrapperFile)) fs.unlinkSync(wrapperFile);
            const outputFile = path.join(tempDir, name);
            if (fs.existsSync(outputFile)) fs.unlinkSync(outputFile);
            if (fs.existsSync(tempDir)) {
                const remaining = fs.readdirSync(tempDir);
                if (remaining.length === 0) fs.rmdirSync(tempDir);
            }
        } catch (_) {}
    }

    dispose() {
        if (this.watchDisposable) {
            this.watchDisposable.dispose();
        }
        this.controller.dispose();
    }
}
