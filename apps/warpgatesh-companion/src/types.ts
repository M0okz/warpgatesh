export type CompanionProfile = {
  name: string;
  username: string;
  baseUrl: string;
  warpgateVersion: string | null;
  sshHost: string;
  sshPort: number;
  isDefault: boolean;
};

export type CompanionTarget = {
  alias: string;
  qualifiedAlias: string;
  name: string;
  profile: string;
};

export type CompanionPreferences = {
  syncIntervalSeconds: number;
  launchCompanionAtLogin: boolean;
  defaultProfile: string | null;
};

export type CompanionAlert = {
  id: string;
  kind: "warning" | "error";
  title: string;
  message: string;
  action: "profiles" | null;
};

export type CompanionState = {
  agentRunning: boolean;
  agentSynchronizing: boolean;
  profiles: CompanionProfile[];
  targets: CompanionTarget[];
  lastSyncAgeSeconds: number | null;
  preferences: CompanionPreferences;
  terminalIntegration: TerminalIntegration;
  alerts: CompanionAlert[];
};

export type TerminalIntegration = {
  status: "managed" | "external" | "missing" | "conflict";
  path: string;
};

export type UninstallRequest = {
  deleteUserData: boolean;
  confirmation: string;
};

export type ProfileRequest = {
  name: string;
  baseUrl: string;
  token: string;
  sshHost?: string;
  sshPort?: number;
};

export type ProfileInspection = {
  normalizedBaseUrl: string;
  username: string;
  warpgateVersion: string | null;
  sshHost: string;
  sshPort: number;
  fingerprints: string;
};
