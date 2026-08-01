import { useCallback, useEffect, useState } from "react";
import { getAppSnapshot, describeError } from "../services/tauriCommands";
import { listenForSnapshots } from "../services/tauriEvents";
import type { AppSnapshot } from "../types/appTypes";

export function useAppSnapshot() {
  const [snapshot, setSnapshot] = useState<AppSnapshot | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const replaceSnapshot = useCallback((next: AppSnapshot) => {
    setSnapshot(next);
    setError(null);
  }, []);

  useEffect(() => {
    let active = true;
    let cleanup: (() => void) | undefined;

    async function connect() {
      try {
        cleanup = await listenForSnapshots((next) => {
          if (active) replaceSnapshot(next);
        });
        if (!active) {
          cleanup();
          return;
        }
        const initial = await getAppSnapshot();
        if (active) replaceSnapshot(initial);
      } catch (reason) {
        if (active) setError(describeError(reason));
      } finally {
        if (active) setLoading(false);
      }
    }

    void connect();
    return () => {
      active = false;
      cleanup?.();
    };
  }, [replaceSnapshot]);

  return { snapshot, loading, error, setError, replaceSnapshot };
}

