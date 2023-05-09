import * as path from 'path';

import * as cp from 'child_process';
import {
	downloadAndUnzipVSCode,
	resolveCliArgsFromVSCodeExecutablePath,
	runTests,
} from '@vscode/test-electron';

async function main() {
	try {
		const vscodeExecutablePath = await downloadAndUnzipVSCode();
		// const [cliPath, ...args] = resolveCliArgsFromVSCodeExecutablePath(vscodeExecutablePath);

		// // Use cp.spawn / cp.exec for custom setup
		// cp.spawnSync(
		// 	cliPath,
		// 	[...args, '--install-extension', 'rust-lang.rust-analyzer'],
		// 	{
		// 		encoding: 'utf-8',
		// 		stdio: 'inherit'
		// 	}
		// );

		// The folder containing the Extension Manifest package.json
		// Passed to `--extensionDevelopmentPath`
		const extensionDevelopmentPath = path.resolve(__dirname, '../../');

		// The path to test runner
		// Passed to --extensionTestsPath
		const extensionTestsPath = path.resolve(__dirname, './suite/index');

		// Download VS Code, unzip it and run the integration test
		await runTests({
			// Use the specified `code` executable
			vscodeExecutablePath,
			extensionDevelopmentPath,
			extensionTestsPath,
		});
	} catch (err) {
		console.error('Failed to run tests', err);
		process.exit(1);
	}
}

main();
