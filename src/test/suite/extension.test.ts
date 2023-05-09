import * as assert from 'assert';

// You can import and use all API from the 'vscode' module
// as well as import your extension to test it
import * as vscode from 'vscode';
import * as rustext from '../../extension';



suite('My Rust Analyzer integration', () => {
	let uri = vscode.Uri.file('/Users/ViktorL/projects/github/rust-quick-fixes/src/test/suite/file.rs');
	// rustext.activate();

	test('Adding Debug trait to struct', async () => {
		let ext = vscode.extensions.all.find(ext => {
			if (ext.id.includes(".rust-quick-fixes")) {
				return ext;
			}
			return false;
		});

		await ext?.activate();


		// Open a Rust file
		let document = await vscode.workspace.openTextDocument(uri);
		await vscode.window.showTextDocument(document);
		let editor = vscode.window.activeTextEditor;

		let start = new vscode.Position(0, 0);
		// console.log("please show up", editor, range);
		if (editor) {
			// Move the cursor to the struct definition
			const text = editor.document.getText();
			let end = start.translate(0, text.length);
			const range = new vscode.Range(start, end);

			editor.selection = new vscode.Selection(start, end);

			const codeActions: [{ title: String, kind: { value: string } }] = await vscode.commands.executeCommand("vscode.executeCodeActionProvider", document.uri, range, vscode.CodeActionKind.QuickFix.value, 3);

			await vscode.commands.executeCommand('editor.action.quickFix', codeActions);


			console.log("hello", codeActions, (await vscode.commands.getCommands()).filter(cmd => cmd.includes("rust")));


			await vscode.commands.executeCommand('extension.rustfixer');


			return;
		}

		assert.ifError("Error editor not found");
	});
});