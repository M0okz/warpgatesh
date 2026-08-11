import { invoke } from "@tauri-apps/api/core";
import type { CompanionState } from "./types";

const demoState: CompanionState = {
  agentRunning: true,
  profiles: [
    {
      name: "homeblack",
      username: "gregory.narcin",
      baseUrl: "https://bastion.int.homeblack.fr/",
      isDefault: true,
    },
  ],
  targets: [
    "dmz-nextcloud-01",
    "dmz-gitlab-01",
    "trust-auth-01",
    "trust-dns-01",
    "trust-wazuh-01",
    "pve-dell",
  ].map((name) => ({
    alias: `${name}.homeblack`,
    qualifiedAlias: `${name}.homeblack`,
    name,
    profile: "homeblack",
  })),
  lastSyncAgeSeconds: 74,
};

const isTauri = "__TAURI_INTERNALS__" in window;

export async function getCompanionState(): Promise<CompanionState> {
  if (import.meta.env.DEV && !isTauri) return demoState;
  return invoke<CompanionState>("get_companion_state");
}

export async function synchronizeNow(): Promise<string> {
  if (import.meta.env.DEV && !isTauri) return "Demo synchronization complete";
  return invoke<string>("sync_now");
}

export async function openTarget(alias: string): Promise<void> {
  if (import.meta.env.DEV && !isTauri) return;
  return invoke<void>("open_target", { alias });
}
