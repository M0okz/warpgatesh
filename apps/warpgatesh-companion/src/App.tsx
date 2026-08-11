import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import {
  addProfile,
  getCompanionState,
  inspectProfile,
  installCommandLineTool,
  openTarget,
  openTokenPage,
  removeProfile,
  renewProfileToken,
  savePreferences,
  synchronizeNow,
  uninstallWarpgateSH,
} from "./api";
import type {
  CompanionPreferences,
  CompanionProfile,
  CompanionState,
  CompanionTarget,
  ProfileInspection,
  ProfileRequest,
} from "./types";

type View = "access" | "profiles" | "preferences";

function formatAge(seconds: number | null): string {
  if (seconds === null) return "Jamais synchronisé";
  if (seconds < 60) return "À l’instant";
  const minutes = Math.floor(seconds / 60);
  if (minutes < 60) return `Il y a ${minutes} min`;
  const hours = Math.floor(minutes / 60);
  return `Il y a ${hours} h`;
}

function RouteLine({ running }: { running: boolean }) {
  return (
    <div className="route-line" aria-label={running ? "Agent connecté" : "Agent hors ligne"}>
      <span className="route-node route-node--local">Mac</span>
      <span className="route-track" aria-hidden="true" />
      <span className={`route-node ${running ? "route-node--live" : "route-node--offline"}`}>
        Warpgate
      </span>
      <span className="route-track route-track--quiet" aria-hidden="true" />
      <span className="route-node">Cible</span>
    </div>
  );
}

function TargetRow({ target, onOpen }: { target: CompanionTarget; onOpen: (alias: string) => void }) {
  return (
    <li className="target-row">
      <button className="target-main" type="button" onClick={() => onOpen(target.alias)}>
        <span className="target-name">{target.name}</span>
        <span className="target-alias">{target.alias}</span>
      </button>
      <span className="profile-mark" title={`Profil ${target.profile}`}>
        {target.profile.slice(0, 1).toUpperCase()}
      </span>
    </li>
  );
}

function AccessView({
  state,
  busy,
  onSync,
  onOpen,
  onNavigate,
}: {
  state: CompanionState;
  busy: boolean;
  onSync: () => void;
  onOpen: (alias: string) => void;
  onNavigate: (view: View) => void;
}) {
  const searchInput = useRef<HTMLInputElement>(null);
  const [query, setQuery] = useState("");

  useEffect(() => {
    function focusSearch(event: KeyboardEvent) {
      if (event.metaKey && event.key.toLocaleLowerCase() === "k") {
        event.preventDefault();
        searchInput.current?.focus();
      }
    }
    window.addEventListener("keydown", focusSearch);
    return () => window.removeEventListener("keydown", focusSearch);
  }, []);

  const normalizedQuery = query.trim().toLocaleLowerCase("fr");
  const visibleTargets = useMemo(() => {
    if (!normalizedQuery) return state.targets;
    return state.targets.filter((target) =>
      `${target.name} ${target.alias} ${target.profile}`
        .toLocaleLowerCase("fr")
        .includes(normalizedQuery),
    );
  }, [normalizedQuery, state.targets]);

  return (
    <>
      <section className="connection-panel" aria-label="État de la connexion">
        <RouteLine running={state.agentRunning} />
        <div className="metrics-row">
          <div>
            <span className="metric-value">{state.targets.length}</span>
            <span className="metric-label">cibles</span>
          </div>
          <div>
            <span className="metric-value">{state.profiles.length}</span>
            <span className="metric-label">profils</span>
          </div>
          <div className="metric-sync">
            <span className="metric-value metric-value--text">
              {formatAge(state.lastSyncAgeSeconds)}
            </span>
            <span className="metric-label">dernière synchro</span>
          </div>
        </div>
        <button className="sync-button" type="button" disabled={busy || !state.agentRunning} onClick={onSync}>
          <span className={busy ? "sync-glyph sync-glyph--busy" : "sync-glyph"} aria-hidden="true">
            ↻
          </span>
          {busy ? "Synchronisation…" : "Synchroniser maintenant"}
        </button>
      </section>

      {state.alerts.map((alert) => (
        <div className={`health-alert health-alert--${alert.kind}`} role="alert" key={alert.id}>
          <div>
            <strong>{alert.title}</strong>
            <p>{alert.message}</p>
          </div>
          {alert.action ? (
            <button type="button" onClick={() => onNavigate(alert.action as View)}>
              Corriger
            </button>
          ) : null}
        </div>
      ))}

      <section className="targets-panel" aria-labelledby="targets-title">
        <div className="section-heading">
          <div>
            <p className="section-kicker">Accès SSH</p>
            <h2 id="targets-title">Cibles disponibles</h2>
          </div>
          <span className="result-count">{visibleTargets.length}</span>
        </div>
        <label className="search-field">
          <span className="sr-only">Rechercher une cible</span>
          <span aria-hidden="true">⌕</span>
          <input
            ref={searchInput}
            type="search"
            value={query}
            onChange={(event) => setQuery(event.target.value)}
            placeholder="Nom, alias ou profil"
            autoComplete="off"
          />
          <kbd>⌘ K</kbd>
        </label>
        {visibleTargets.length === 0 ? (
          <p className="empty-state">Aucune cible ne correspond à cette recherche.</p>
        ) : (
          <ul className="target-list">
            {visibleTargets.map((target) => (
              <TargetRow key={target.qualifiedAlias} target={target} onOpen={onOpen} />
            ))}
          </ul>
        )}
      </section>
    </>
  );
}

function ProfileCard({
  profile,
  busy,
  onRenew,
  onRemove,
}: {
  profile: CompanionProfile;
  busy: boolean;
  onRenew: (name: string, token: string) => Promise<boolean>;
  onRemove: (name: string) => Promise<void>;
}) {
  const [renewing, setRenewing] = useState(false);
  const [token, setToken] = useState("");

  return (
    <article className="profile-card">
      <div className="profile-card__heading">
        <span className="profile-avatar">{profile.name.slice(0, 1).toUpperCase()}</span>
        <div>
          <h3>{profile.name}</h3>
          <p>{profile.username}</p>
        </div>
        {profile.isDefault ? <span className="default-label">Par défaut</span> : null}
      </div>
      <dl className="profile-details">
        <div><dt>API</dt><dd>{profile.baseUrl}</dd></div>
        <div><dt>SSH</dt><dd>{profile.sshHost}:{profile.sshPort}</dd></div>
        <div><dt>Version</dt><dd>{profile.warpgateVersion ?? "Inconnue"}</dd></div>
      </dl>
      {renewing ? (
        <form
          className="inline-form"
          onSubmit={(event) => {
            event.preventDefault();
            void onRenew(profile.name, token).then((saved) => {
              if (saved) {
                setToken("");
                setRenewing(false);
              }
            });
          }}
        >
          <label><span>Nouveau jeton API</span><input type="password" value={token} onChange={(event) => setToken(event.target.value)} autoFocus /></label>
          <div className="button-row">
            <button className="button-secondary" type="button" onClick={() => setRenewing(false)}>Annuler</button>
            <button className="button-primary" type="submit" disabled={busy || !token.trim()}>Valider</button>
          </div>
        </form>
      ) : (
        <div className="card-actions">
          <button type="button" onClick={() => setRenewing(true)}>Renouveler le jeton</button>
          <button className="text-danger" type="button" onClick={() => void onRemove(profile.name)}>Supprimer</button>
        </div>
      )}
    </article>
  );
}

function ProfilesView({
  profiles,
  busy,
  onChanged,
  runAction,
}: {
  profiles: CompanionProfile[];
  busy: boolean;
  onChanged: () => Promise<void>;
  runAction: (action: () => Promise<void>, success: string) => Promise<boolean>;
}) {
  const [adding, setAdding] = useState(false);
  const [request, setRequest] = useState<ProfileRequest>({ name: "", baseUrl: "", token: "" });
  const [inspection, setInspection] = useState<ProfileInspection | null>(null);
  const [advanced, setAdvanced] = useState(false);

  function update(field: keyof ProfileRequest, value: string | number | undefined) {
    setInspection(null);
    setRequest((current) => ({ ...current, [field]: value }));
  }

  async function inspect() {
    await runAction(async () => setInspection(await inspectProfile(request)), "Clés SSH récupérées. Vérifiez-les avant de confirmer.");
  }

  async function save() {
    await runAction(async () => {
      await addProfile(request);
      setAdding(false);
      setInspection(null);
      setRequest({ name: "", baseUrl: "", token: "" });
      await onChanged();
    }, "Profil ajouté et synchronisé.");
  }

  async function renew(name: string, token: string): Promise<boolean> {
    return runAction(async () => {
      await renewProfileToken(name, token);
      await onChanged();
    }, "Jeton renouvelé et profil synchronisé.");
  }

  async function remove(name: string) {
    if (!window.confirm(`Supprimer le profil « ${name} » et ses alias SSH ?`)) return;
    await runAction(async () => {
      await removeProfile(name);
      await onChanged();
    }, "Profil supprimé.");
  }

  return (
    <section className="page-panel" aria-labelledby="profiles-title">
      <div className="page-heading">
        <div><p className="section-kicker">Instances</p><h2 id="profiles-title">Profils Warpgate</h2></div>
        <button className="button-primary button-compact" type="button" onClick={() => setAdding((value) => !value)}>{adding ? "Fermer" : "+ Ajouter"}</button>
      </div>

      {adding ? (
        <form className="profile-form" onSubmit={(event) => { event.preventDefault(); void inspect(); }}>
          <div className="form-grid">
            <label><span>Nom du profil</span><input value={request.name} onChange={(event) => update("name", event.target.value)} placeholder="mon-instance" autoComplete="off" /></label>
            <label><span>URL Warpgate</span><input type="url" value={request.baseUrl} onChange={(event) => update("baseUrl", event.target.value)} placeholder="https://bastion.example.org" /></label>
          </div>
          <label><span>Jeton API personnel</span><input type="password" value={request.token} onChange={(event) => update("token", event.target.value)} /></label>
          <div className="form-help">
            <button type="button" onClick={() => void runAction(() => openTokenPage(request.baseUrl), "Page des jetons ouverte dans votre navigateur.")} disabled={!request.baseUrl}>Créer un jeton dans Warpgate ↗</button>
            <button type="button" onClick={() => setAdvanced((value) => !value)}>{advanced ? "Masquer l’adresse SSH" : "Adresse SSH différente ?"}</button>
          </div>
          {advanced ? (
            <div className="form-grid form-grid--endpoint">
              <label><span>Hôte SSH</span><input value={request.sshHost ?? ""} onChange={(event) => update("sshHost", event.target.value || undefined)} placeholder="Auto-détecté" /></label>
              <label><span>Port</span><input type="number" min="1" max="65535" value={request.sshPort ?? ""} onChange={(event) => update("sshPort", event.target.value ? Number(event.target.value) : undefined)} placeholder="2222" /></label>
            </div>
          ) : null}
          {inspection ? (
            <div className="fingerprint-box">
              <strong>Identité SSH de {inspection.sshHost}:{inspection.sshPort}</strong>
              <pre>{inspection.fingerprints.trim()}</pre>
              <p>Comparez ces empreintes à une source de confiance avant de les épingler.</p>
              <button className="button-primary" type="button" disabled={busy} onClick={() => void save()}>Faire confiance et ajouter</button>
            </div>
          ) : (
            <button className="button-primary" type="submit" disabled={busy || !request.name || !request.baseUrl || !request.token}>Vérifier la connexion</button>
          )}
        </form>
      ) : null}

      <div className="profile-stack">
        {profiles.length === 0 ? <p className="empty-state">Aucun profil configuré.</p> : profiles.map((profile) => (
          <ProfileCard key={profile.name} profile={profile} busy={busy} onRenew={renew} onRemove={remove} />
        ))}
      </div>
    </section>
  );
}

function PreferencesView({
  state,
  busy,
  onSave,
  onInstallCli,
  onUninstall,
}: {
  state: CompanionState;
  busy: boolean;
  onSave: (preferences: CompanionPreferences) => Promise<void>;
  onInstallCli: () => Promise<void>;
  onUninstall: (deleteUserData: boolean, confirmation: string) => Promise<void>;
}) {
  const [draft, setDraft] = useState(state.preferences);
  const [showUninstall, setShowUninstall] = useState(false);
  const [deleteUserData, setDeleteUserData] = useState(false);
  const [uninstallConfirmation, setUninstallConfirmation] = useState("");

  useEffect(() => setDraft(state.preferences), [
    state.preferences.defaultProfile,
    state.preferences.launchCompanionAtLogin,
    state.preferences.syncIntervalSeconds,
  ]);

  const terminal = state.terminalIntegration;
  const terminalLabels = {
    managed: "Installée par WarpgateSH",
    external: "Déjà disponible",
    missing: "Non installée",
    conflict: "Conflit détecté",
  } as const;

  return (
    <section className="page-panel" aria-labelledby="preferences-title">
      <div className="page-heading"><div><p className="section-kicker">Comportement</p><h2 id="preferences-title">Préférences</h2></div></div>
      <form className="preferences-form" onSubmit={(event) => { event.preventDefault(); void onSave(draft); }}>
        <label className="setting-row">
          <span><strong>Synchronisation automatique</strong><small>Cadence utilisée par l’agent, même lorsque l’app est fermée.</small></span>
          <select value={draft.syncIntervalSeconds} onChange={(event) => setDraft((value) => ({ ...value, syncIntervalSeconds: Number(event.target.value) }))}>
            <option value={60}>Chaque minute</option><option value={300}>Toutes les 5 min</option><option value={900}>Toutes les 15 min</option><option value={1800}>Toutes les 30 min</option><option value={3600}>Chaque heure</option>
          </select>
        </label>
        <label className="setting-row">
          <span><strong>Profil par défaut</strong><small>Ses cibles restent accessibles avec leur alias court.</small></span>
          <select value={draft.defaultProfile ?? ""} onChange={(event) => setDraft((value) => ({ ...value, defaultProfile: event.target.value || null }))}>
            {state.profiles.length === 0 ? <option value="">Aucun</option> : state.profiles.map((profile) => <option key={profile.name} value={profile.name}>{profile.name}</option>)}
          </select>
        </label>
        <label className="setting-row setting-row--toggle">
          <span><strong>Ouvrir à la connexion</strong><small>Place WarpgateSH dans la barre des menus au démarrage du Mac.</small></span>
          <input type="checkbox" checked={draft.launchCompanionAtLogin} onChange={(event) => setDraft((value) => ({ ...value, launchCompanionAtLogin: event.target.checked }))} />
        </label>
        <div className="setting-row setting-row--terminal">
          <span>
            <strong>Intégration terminal</strong>
            <small>
              {terminal.status === "external"
                ? "Une installation existante fournit déjà la commande warpgatesh."
                : terminal.status === "conflict"
                  ? "Ce chemin est occupé par une commande qui n’appartient pas à WarpgateSH."
                  : "Rend la CLI principale disponible sans modifier votre shell ni son PATH."}
            </small>
          </span>
          <div className="terminal-control" aria-live="polite">
            <span className={`terminal-status terminal-status--${terminal.status}`}>
              <span aria-hidden="true" />
              {terminalLabels[terminal.status]}
            </span>
            <code title={terminal.path}>{terminal.path}</code>
            {terminal.status === "missing" ? (
              <button className="button-secondary" type="button" disabled={busy} onClick={() => void onInstallCli()}>
                Installer la CLI
              </button>
            ) : null}
          </div>
        </div>
        <button className="button-primary preferences-save" type="submit" disabled={busy}>Enregistrer les préférences</button>
      </form>
      <div className="danger-zone">
        <div>
          <p className="section-kicker">Désinstallation</p>
          <h2>Retirer WarpgateSH</h2>
          <p>L’agent sera arrêté, la CLI installée par l’app sera retirée et l’application sera placée dans la Corbeille.</p>
        </div>
        {!showUninstall ? (
          <button className="button-danger-outline" type="button" disabled={busy} onClick={() => setShowUninstall(true)}>
            Désinstaller…
          </button>
        ) : (
          <div className="uninstall-confirmation">
            <label className="destructive-option">
              <input type="checkbox" checked={deleteUserData} onChange={(event) => setDeleteUserData(event.target.checked)} />
              <span><strong>Supprimer aussi mes données</strong><small>Efface les profils, jetons du Trousseau, instantanés et fichiers SSH gérés. Cette action est irréversible.</small></span>
            </label>
            <label className="confirmation-field">
              <span>Saisissez <strong>DÉSINSTALLER</strong> pour confirmer</span>
              <input autoComplete="off" value={uninstallConfirmation} onChange={(event) => setUninstallConfirmation(event.target.value)} />
            </label>
            <div className="uninstall-actions">
              <button className="button-secondary" type="button" disabled={busy} onClick={() => { setShowUninstall(false); setDeleteUserData(false); setUninstallConfirmation(""); }}>Annuler</button>
              <button className="button-danger" type="button" disabled={busy || uninstallConfirmation.trim() !== "DÉSINSTALLER"} onClick={() => void onUninstall(deleteUserData, uninstallConfirmation)}>
                {deleteUserData ? "Tout supprimer" : "Désinstaller et conserver les données"}
              </button>
            </div>
          </div>
        )}
      </div>
    </section>
  );
}

export default function App() {
  const refreshing = useRef(false);
  const [view, setView] = useState<View>("access");
  const [state, setState] = useState<CompanionState | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);

  const refresh = useCallback(async () => {
    if (refreshing.current) return;
    refreshing.current = true;
    try {
      setState(await getCompanionState());
    } catch (reason) {
      setError(String(reason));
    } finally {
      refreshing.current = false;
    }
  }, []);

  useEffect(() => {
    void refresh();
    const interval = window.setInterval(() => {
      if (document.visibilityState === "visible") void refresh();
    }, 5_000);
    function onVisibilityChange() {
      if (document.visibilityState === "visible") void refresh();
    }
    document.addEventListener("visibilitychange", onVisibilityChange);
    return () => {
      window.clearInterval(interval);
      document.removeEventListener("visibilitychange", onVisibilityChange);
    };
  }, [refresh]);

  async function runAction(action: () => Promise<void>, success: string): Promise<boolean> {
    setBusy(true);
    setError(null);
    setNotice(null);
    try {
      await action();
      setNotice(success);
      return true;
    } catch (reason) {
      setError(String(reason));
      return false;
    } finally {
      setBusy(false);
    }
  }

  async function handleSync() {
    await runAction(async () => {
      await synchronizeNow();
      await refresh();
    }, "Synchronisation demandée à l’agent.");
  }

  async function handleOpen(alias: string) {
    setError(null);
    try { await openTarget(alias); } catch (reason) { setError(String(reason)); }
  }

  async function handlePreferences(preferences: CompanionPreferences) {
    await runAction(async () => {
      await savePreferences(preferences);
      await refresh();
    }, "Préférences enregistrées.");
  }

  async function handleInstallCli() {
    await runAction(async () => {
      await installCommandLineTool();
      await refresh();
    }, "La commande warpgatesh est disponible dans le terminal.");
  }

  async function handleUninstall(deleteUserData: boolean, confirmation: string) {
    setBusy(true);
    setError(null);
    setNotice(null);
    try {
      await uninstallWarpgateSH({ deleteUserData, confirmation });
    } catch (reason) {
      setError(String(reason));
      setBusy(false);
    }
  }

  const running = state?.agentRunning ?? false;
  const defaultProfile = state?.profiles.find((profile) => profile.isDefault);

  return (
    <div className="app-shell">
      <header className="app-header">
        <div><p className="eyebrow">WarpgateSH</p><h1>{view === "access" ? "Vos accès, à jour." : view === "profiles" ? "Vos instances." : "À votre rythme."}</h1></div>
        <span className={`status-pill ${running ? "status-pill--live" : "status-pill--offline"}`}><span className="status-dot" aria-hidden="true" />{running ? "Agent actif" : "Agent arrêté"}</span>
      </header>

      <nav className="view-tabs" aria-label="Sections">
        {([['access', 'Accès'], ['profiles', 'Profils'], ['preferences', 'Préférences']] as const).map(([id, label]) => (
          <button key={id} type="button" className={view === id ? "is-active" : ""} onClick={() => setView(id)}>{label}</button>
        ))}
      </nav>

      {error ? <div className="error-banner" role="alert"><span>Action impossible</span><p>{error}</p></div> : null}
      {notice ? <div className="notice-banner" role="status">{notice}</div> : null}

      <main>
        {state === null ? <p className="loading-state">Lecture de l’état local…</p> : null}
        {state && view === "access" ? <AccessView state={state} busy={busy} onSync={() => void handleSync()} onOpen={(alias) => void handleOpen(alias)} onNavigate={setView} /> : null}
        {state && view === "profiles" ? <ProfilesView profiles={state.profiles} busy={busy} onChanged={refresh} runAction={runAction} /> : null}
        {state && view === "preferences" ? <PreferencesView state={state} busy={busy} onSave={handlePreferences} onInstallCli={handleInstallCli} onUninstall={handleUninstall} /> : null}
      </main>

      <footer className="profile-footer">
        <span className="profile-avatar" aria-hidden="true">{defaultProfile?.name.slice(0, 1).toUpperCase() ?? "—"}</span>
        <span><strong>{defaultProfile?.name ?? "Aucun profil"}</strong><small>{defaultProfile?.username ?? "Ajoutez une instance pour commencer"}</small></span>
        <span className="default-label">Par défaut</span>
      </footer>
    </div>
  );
}
