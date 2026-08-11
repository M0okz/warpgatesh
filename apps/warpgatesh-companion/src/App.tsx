import { useEffect, useMemo, useRef, useState } from "react";
import { getCompanionState, openTarget, synchronizeNow } from "./api";
import type { CompanionState, CompanionTarget } from "./types";

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

export default function App() {
  const searchInput = useRef<HTMLInputElement>(null);
  const [state, setState] = useState<CompanionState | null>(null);
  const [query, setQuery] = useState("");
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let active = true;
    getCompanionState()
      .then((nextState) => {
        if (active) setState(nextState);
      })
      .catch((reason: unknown) => {
        if (active) setError(String(reason));
      });
    return () => {
      active = false;
    };
  }, []);

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
    if (!state || normalizedQuery.length === 0) return state?.targets ?? [];
    return state.targets.filter((target) =>
      `${target.name} ${target.alias} ${target.profile}`
        .toLocaleLowerCase("fr")
        .includes(normalizedQuery),
    );
  }, [normalizedQuery, state]);

  async function handleSync() {
    setBusy(true);
    setError(null);
    try {
      await synchronizeNow();
      setState(await getCompanionState());
    } catch (reason) {
      setError(String(reason));
    } finally {
      setBusy(false);
    }
  }

  async function handleOpen(alias: string) {
    setError(null);
    try {
      await openTarget(alias);
    } catch (reason) {
      setError(String(reason));
    }
  }

  const running = state?.agentRunning ?? false;
  const defaultProfile = state?.profiles.find((profile) => profile.isDefault);

  return (
    <div className="app-shell">
      <header className="app-header">
        <div>
          <p className="eyebrow">WarpgateSH</p>
          <h1>Vos accès, à jour.</h1>
        </div>
        <span className={`status-pill ${running ? "status-pill--live" : "status-pill--offline"}`}>
          <span className="status-dot" aria-hidden="true" />
          {running ? "Agent actif" : "Agent arrêté"}
        </span>
      </header>

      <main>
        <section className="connection-panel" aria-labelledby="connection-title">
          <h2 id="connection-title" className="sr-only">
            État de la connexion
          </h2>
          <RouteLine running={running} />
          <div className="metrics-row">
            <div>
              <span className="metric-value">{state?.targets.length ?? "—"}</span>
              <span className="metric-label">cibles</span>
            </div>
            <div>
              <span className="metric-value">{state?.profiles.length ?? "—"}</span>
              <span className="metric-label">profil</span>
            </div>
            <div className="metric-sync">
              <span className="metric-value metric-value--text">
                {formatAge(state?.lastSyncAgeSeconds ?? null)}
              </span>
              <span className="metric-label">dernière synchro</span>
            </div>
          </div>
          <button className="sync-button" type="button" disabled={busy || !running} onClick={handleSync}>
            <span className={busy ? "sync-glyph sync-glyph--busy" : "sync-glyph"} aria-hidden="true">
              ↻
            </span>
            {busy ? "Synchronisation…" : "Synchroniser maintenant"}
          </button>
        </section>

        {error ? (
          <div className="error-banner" role="alert">
            <span>Action impossible</span>
            <p>{error}</p>
          </div>
        ) : null}

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

          {state === null && error === null ? <p className="empty-state">Lecture de l’instantané local…</p> : null}
          {state !== null && visibleTargets.length === 0 ? (
            <p className="empty-state">Aucune cible ne correspond à cette recherche.</p>
          ) : null}
          {visibleTargets.length > 0 ? (
            <ul className="target-list">
              {visibleTargets.map((target) => (
                <TargetRow key={target.qualifiedAlias} target={target} onOpen={handleOpen} />
              ))}
            </ul>
          ) : null}
        </section>
      </main>

      <footer className="profile-footer">
        <span className="profile-avatar" aria-hidden="true">
          {defaultProfile?.name.slice(0, 1).toUpperCase() ?? "—"}
        </span>
        <span>
          <strong>{defaultProfile?.name ?? "Aucun profil"}</strong>
          <small>{defaultProfile?.username ?? "Configurez la CLI pour commencer"}</small>
        </span>
        <span className="default-label">Par défaut</span>
      </footer>
    </div>
  );
}
