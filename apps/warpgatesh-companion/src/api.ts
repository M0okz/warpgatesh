import { invoke } from "@tauri-apps/api/core";
import type {
  CompanionPreferences,
  CompanionState,
  DiagnosticsExport,
  DiagnosticsPreview,
  ProfileInspection,
  ProfileRequest,
  TerminalIntegration,
  UninstallRequest,
  UpdateStatus,
} from "./types";

const demoState: CompanionState = {
  agentRunning: true,
  agentSynchronizing: false,
  profiles: [
    {
      name: "homeblack",
      username: "gregory.narcin",
      baseUrl: "https://bastion.int.homeblack.fr/",
      warpgateVersion: "0.27.1",
      sshHost: "bastion.int.homeblack.fr",
      sshPort: 2222,
      isDefault: true,
    },
  ],
  targets: ["dmz-nextcloud-01", "dmz-gitlab-01", "trust-auth-01", "trust-dns-01"].map(
    (name) => ({
      alias: `${name}.homeblack`,
      qualifiedAlias: `${name}.homeblack`,
      name,
      profile: "homeblack",
    }),
  ),
  lastSyncAgeSeconds: 74,
  preferences: {
    syncIntervalSeconds: 300,
    launchCompanionAtLogin: false,
    defaultProfile: "homeblack",
  },
  terminalIntegration: {
    status: "missing",
    path: "/usr/local/bin/warpgatesh",
  },
  update: {
    phase: "available",
    channel: "direct",
    currentVersion: "0.1.9",
    availableVersion: "0.1.10",
    notes: "Mise à jour signée avec amélioration du menu et de la synchronisation.",
    checkedAtEpochSeconds: Math.floor(Date.now() / 1000),
    progressPercent: null,
    message: null,
  },
  alerts: [],
};

const isTauri = "__TAURI_INTERNALS__" in window;
const isDemo = import.meta.env.DEV && !isTauri;

export async function getCompanionState(): Promise<CompanionState> {
  if (isDemo) return demoState;
  return invoke<CompanionState>("get_companion_state");
}

export async function synchronizeNow(): Promise<string> {
  if (isDemo) return "Demo synchronization complete";
  return invoke<string>("sync_now");
}

export async function savePreferences(preferences: CompanionPreferences): Promise<void> {
  if (isDemo) return;
  return invoke<void>("save_preferences", { preferences });
}

export async function openTokenPage(baseUrl: string): Promise<void> {
  if (isDemo) return;
  return invoke<void>("open_token_page_for", { baseUrl });
}

export async function inspectProfile(request: ProfileRequest): Promise<ProfileInspection> {
  if (isDemo) {
    return {
      normalizedBaseUrl: request.baseUrl,
      username: "gregory.narcin",
      warpgateVersion: "0.27.1",
      sshHost: request.sshHost || "bastion.example.org",
      sshPort: request.sshPort || 2222,
      fingerprints: "256 SHA256:example ED25519\n3072 SHA256:example RSA",
    };
  }
  return invoke<ProfileInspection>("inspect_profile", { request });
}

export async function addProfile(request: ProfileRequest): Promise<void> {
  if (isDemo) return;
  return invoke<void>("add_profile", { request });
}

export async function renewProfileToken(name: string, token: string): Promise<void> {
  if (isDemo) return;
  return invoke<void>("renew_profile_token", { name, token });
}

export async function removeProfile(name: string): Promise<void> {
  if (isDemo) return;
  return invoke<void>("remove_profile", { name });
}

export async function openTarget(alias: string): Promise<void> {
  if (isDemo) return;
  return invoke<void>("open_target", { alias });
}

export async function installCommandLineTool(): Promise<TerminalIntegration> {
  if (isDemo) return demoState.terminalIntegration;
  return invoke<TerminalIntegration>("install_command_line_tool");
}

export async function uninstallWarpgateSH(request: UninstallRequest): Promise<void> {
  if (isDemo) return;
  return invoke<void>("uninstall_warpgatesh", { request });
}

export async function checkForUpdates(): Promise<UpdateStatus> {
  if (isDemo) return demoState.update;
  return invoke<UpdateStatus>("check_for_updates");
}

export async function installUpdate(): Promise<void> {
  if (isDemo) return;
  return invoke<void>("install_update");
}

export async function previewDiagnostics(): Promise<DiagnosticsPreview> {
  if (isDemo) {
    return {
      logDirectory: "~/Library/Logs/WarpgateSH",
      retentionDays: 7,
      totalBytes: 2840,
      totalEvents: 18,
      files: [
        { name: "agent-2026-08-13.jsonl", bytes: 2240, events: 14 },
        { name: "companion-2026-08-13.jsonl", bytes: 600, events: 4 },
      ],
    };
  }
  return invoke<DiagnosticsPreview>("preview_diagnostics");
}

export async function exportDiagnostics(): Promise<DiagnosticsExport> {
  if (isDemo) return { path: "~/Downloads/WarpgateSH-diagnostics-demo.zip" };
  return invoke<DiagnosticsExport>("export_diagnostics");
}
