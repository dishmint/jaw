import { ExtensionContext, workspace } from "vscode";
import {
  LanguageClient,
  LanguageClientOptions,
  ServerOptions,
} from "vscode-languageclient/node";

let client: LanguageClient | undefined;

const JAW_BOLD_SCOPES = [
  { scope: "variable.other.jaw", settings: { fontStyle: "bold" } },
  { scope: "variable.other.definition.jaw", settings: { fontStyle: "bold" } },
  { scope: "constant.numeric.step.jaw", settings: { fontStyle: "bold" } },
  { scope: "punctuation.definition.step.jaw", settings: { fontStyle: "bold" } },
  { scope: "markup.italic.log-title.jaw", settings: { fontStyle: "italic" } },
];

async function ensureBoldStyles() {
  const config = workspace.getConfiguration("editor");
  const current = config.get<any>("tokenColorCustomizations") || {};
  const existingRules: any[] = current.textMateRules || [];

  const existingScopes = new Set(existingRules.map((r: any) => r.scope));
  const missing = JAW_BOLD_SCOPES.filter((r) => !existingScopes.has(r.scope));

  if (missing.length > 0) {
    const updated = {
      ...current,
      textMateRules: [...existingRules, ...missing],
    };
    await config.update(
      "tokenColorCustomizations",
      updated,
      true
    );
  }
}

export function activate(context: ExtensionContext) {
  ensureBoldStyles();

  const config = workspace.getConfiguration("jaw");
  const serverPath = config.get<string>("server.path") || "jaw-lsp";

  const serverOptions: ServerOptions = {
    run: { command: serverPath },
    debug: { command: serverPath },
  };

  const clientOptions: LanguageClientOptions = {
    documentSelector: [{ scheme: "file", language: "jaw" }],
  };

  client = new LanguageClient(
    "jaw-lsp",
    "JAW Language Server",
    serverOptions,
    clientOptions
  );

  client.start();
}

export function deactivate(): Thenable<void> | undefined {
  if (client) {
    return client.stop();
  }
  return undefined;
}
