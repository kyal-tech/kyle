import * as vscode from 'vscode';
import * as path from 'path';

function findKyBinary(): string | null {
    const config = vscode.workspace.getConfiguration('ky');
    const configured = config.get<string>('kycPath');
    if (configured && configured !== 'ky') {
        if (require('fs').existsSync(configured)) return configured;
    }
    return 'ky';
}

export class KyleTaskProvider implements vscode.TaskProvider {
    async provideTasks(): Promise<vscode.Task[]> {
        const tasks: vscode.Task[] = [];

        const workspaceFolders = vscode.workspace.workspaceFolders;
        if (!workspaceFolders) return tasks;

        for (const folder of workspaceFolders) {
            const klFiles = await vscode.workspace.findFiles('**/*.ky', '**/node_modules/**');
            if (klFiles.length === 0) continue;

            const uniqueFiles = new Set<string>();
            for (const file of klFiles) {
                const filePath = file.fsPath;
                if (uniqueFiles.has(filePath)) continue;
                uniqueFiles.add(filePath);

                const relativePath = path.relative(folder.uri.fsPath, filePath);

                tasks.push(this.makeTask('Run', 'run', filePath, relativePath, folder));
                tasks.push(this.makeTask('Build', 'build', filePath, relativePath, folder));
                tasks.push(this.makeTask('Check', 'check', filePath, relativePath, folder));
            }
        }

        return tasks;
    }

    private makeTask(
        kind: string,
        subcommand: string,
        filePath: string,
        label: string,
        folder: vscode.WorkspaceFolder
    ): vscode.Task {
        const kyPath = findKyBinary() || 'ky';
        const execution = new vscode.ProcessExecution(kyPath, [subcommand, filePath]);
        const task = new vscode.Task(
            { type: 'ky', subcommand },
            folder,
            `${kind}: ${label}`,
            'KL',
            execution
        );
        task.group = subcommand === 'run'
            ? vscode.TaskGroup.Build
            : subcommand === 'check'
                ? vscode.TaskGroup.Test
                : vscode.TaskGroup.Rebuild;
        task.problemMatchers = [''];
        return task;
    }

    async resolveTask(task: vscode.Task): Promise<vscode.Task | undefined> {
        return task;
    }
}
