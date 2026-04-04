import { ExtensionContext, workspace } from "vscode";

const JAW_BOLD_SCOPES = [
  { scope: "variable.other.jaw", settings: { fontStyle: "bold" } },
  { scope: "variable.other.definition.jaw", settings: { fontStyle: "bold" } },
  { scope: "constant.numeric.step.jaw", settings: { fontStyle: "bold" } },
  { scope: "punctuation.definition.step.jaw", settings: { fontStyle: "bold" } },
];

async function ensureBoldStyles() {
  const config = workspace.getConfiguration("editor");
  const current = config.get<any>("tokenColorCustomizations") || {};
  const existingRules: any[] = current.textMateRules || [];

  // Check if our rules are already present
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
      true // global (user) settings
    );
  }
}

export function activate(context: ExtensionContext) {
  ensureBoldStyles();
}

export function deactivate(): void {}
