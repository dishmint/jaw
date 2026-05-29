import {
  ColorThemeKind,
  ExtensionContext,
  window,
  workspace,
} from "vscode";
import {
  LanguageClient,
  LanguageClientOptions,
  ServerOptions,
} from "vscode-languageclient/node";

let client: LanguageClient | undefined;

// The [•] log marker needs an explicit color that reads in both light and dark
// themes, so it can't be a single hex. Pick per theme kind.
const LOG_MARKER_DARK = "#D7BA7D";
const LOG_MARKER_LIGHT = "#B45309";

function jawTokenRules(kind: ColorThemeKind) {
  const isLight =
    kind === ColorThemeKind.Light || kind === ColorThemeKind.HighContrastLight;
  const logMarker = isLight ? LOG_MARKER_LIGHT : LOG_MARKER_DARK;
  return [
    { scope: "variable.other.jaw", settings: { fontStyle: "bold" } },
    { scope: "variable.other.definition.jaw", settings: { fontStyle: "bold" } },
    { scope: "constant.numeric.step.jaw", settings: { fontStyle: "bold" } },
    { scope: "punctuation.definition.step.jaw", settings: { fontStyle: "bold" } },
    { scope: "markup.italic.log-title.jaw", settings: { fontStyle: "italic" } },
    { scope: "keyword.other.log.jaw", settings: { foreground: logMarker } },
    // Note body/title carry `keyword.other.note.jaw` in the grammar too, so the
    // foreground falls through to the marker's theme color — these rules only add
    // the bold weight.
    { scope: "markup.bold.note.jaw", settings: { fontStyle: "bold" } },
    { scope: "markup.bold.note-title.jaw", settings: { fontStyle: "bold" } },
  ];
}

async function ensureBoldStyles() {
  const desired = jawTokenRules(window.activeColorTheme.kind);

  const config = workspace.getConfiguration("editor");
  const current = config.get<any>("tokenColorCustomizations") || {};
  const existingRules: any[] = current.textMateRules || [];

  const nonJawRules = existingRules.filter(
    (r: any) => typeof r.scope !== "string" || !r.scope.endsWith(".jaw")
  );
  const reconciled = [...nonJawRules, ...desired];

  if (JSON.stringify(reconciled) === JSON.stringify(existingRules)) {
    return;
  }

  const updated = { ...current, textMateRules: reconciled };
  await config.update("tokenColorCustomizations", updated, true);
}

export function activate(context: ExtensionContext) {
  ensureBoldStyles();

  // Re-apply when the user switches between light and dark themes so the log
  // marker picks up the matching amber without a reload.
  context.subscriptions.push(
    window.onDidChangeActiveColorTheme(() => ensureBoldStyles())
  );

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
