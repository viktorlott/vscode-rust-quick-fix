import * as vscode from 'vscode';

import {
	CloseAction,
	ErrorAction,
	Executable,
	LanguageClient,
	LanguageClientOptions,
	Message,
	ServerOptions,
} from "vscode-languageclient/node";


class Client extends LanguageClient {
	private context: vscode.ExtensionContext;

	private static options(_: vscode.ExtensionContext): { serverOptions: ServerOptions, clientOptions: LanguageClientOptions } {
		const serverPath =
			process.env.SERVER_URL || vscode.workspace.getConfiguration("rust-fixer-lsp").get("serverPath") as string;

		const executable: Executable = {
			command: serverPath,
			options: {
				env: {
					RUST_LOG: "debug"
				}
			}
		};

		const serverOptions: ServerOptions = {
			run: executable,
			debug: { ...executable }
		};

		const clientOptions: LanguageClientOptions = {
			documentSelector: [{ scheme: "file", language: "rust" }],
			traceOutputChannel: vscode.window.createOutputChannel(
				"Rust fixer LSP",
				"rust"
			),
			errorHandler: {
				error(error: Error, message: Message | undefined, count: number | undefined) {
					console.log(error, message);
					return {
						action: ErrorAction.Continue,
						message: error.message,
						handled: false
					};
				},
				closed() {
					console.log("shutdown");
					return {
						action: CloseAction.DoNotRestart,
						message: "Shutdown",
						handled: false
					};
				},
			}
		};

		return {
			serverOptions,
			clientOptions
		};
	}

	constructor(context: vscode.ExtensionContext) {
		const { serverOptions, clientOptions } = Client.options(context);

		super(
			"rust-fixer-lsp",
			"Rust fixer LSP",
			serverOptions,
			clientOptions
		);

		this.context = context;
	}

}

export class GenericFixer implements vscode.CodeActionProvider {
	public static readonly providedCodeActionKinds = [
		vscode.CodeActionKind.QuickFix
	];

	public provideCodeActions(document: vscode.TextDocument, range: vscode.Range): vscode.CodeAction[] | undefined {
		let fixMissingGenerics = this.getStructData(document, range);

		if (!fixMissingGenerics) {
			return;
		}

		fixMissingGenerics.isPreferred = true;

		return [
			fixMissingGenerics,
		];
	}

	private getStructData(document: vscode.TextDocument, range: vscode.Range): vscode.CodeAction | undefined {
		const text = document.getText();
		const start = document.lineAt(0).range;

		const line = document.lineAt(start.start.line); // Get the text on line X.
		let match = text.match(/^\s*(pub|priv|pub\(crate\)|pub\(in [\w:]+\)?)?\s*struct\s+(\w+)(<.*?>)?(\(.*\)|\{.*\})/);

		if (match) {
			let string = match[0] || '';
			// Get the visibility and identifier of the struct
			let visibility = match[1] || '';
			let identifier = match[2] || '';
			let generics = match[3] || '';
			let fields = match[4] || '';

			if (generics) {
				return undefined;
			}

			let identifierIndex = string.indexOf(identifier);
			let identifierRange = line.range.start.translate(0, identifierIndex + identifier.length);

			const fix = new vscode.CodeAction("Fix missing generics", vscode.CodeActionKind.QuickFix);

			fix.edit = new vscode.WorkspaceEdit();
			fix.edit.insert(document.uri, identifierRange, "<T>");
			return fix;
		}

		return undefined;
	}
}




export function activate(context: vscode.ExtensionContext) {
	const client = new Client(context);
	client.start();
}

// export function deactivate(context: vscode.ExtensionContext) {
// 	client?.stop();
// }
