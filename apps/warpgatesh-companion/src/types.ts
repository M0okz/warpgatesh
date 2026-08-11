export type CompanionProfile = {
  name: string;
  username: string;
  baseUrl: string;
  isDefault: boolean;
};

export type CompanionTarget = {
  alias: string;
  qualifiedAlias: string;
  name: string;
  profile: string;
};

export type CompanionState = {
  agentRunning: boolean;
  profiles: CompanionProfile[];
  targets: CompanionTarget[];
  lastSyncAgeSeconds: number | null;
};
