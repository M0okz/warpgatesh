export function createRefreshQueue<T>(
  load: () => Promise<T>,
  onValue: (value: T) => void,
  onError: (reason: unknown) => void,
): (afterMutation?: boolean) => Promise<void> {
  let active: Promise<void> | undefined;
  let pending = false;
  return (afterMutation = false) => {
    if (active) {
      if (afterMutation) pending = true;
      return active;
    }
    pending = true;
    // Start after assigning active, including when load throws synchronously.
    active = Promise.resolve().then(async () => {
      try {
        while (pending) {
          pending = false;
          try {
            const value = await load();
            // A newer request may follow a mutation; don't display its old snapshot.
            if (!pending) onValue(value);
          } catch (reason) {
            if (!pending) onError(reason);
          }
        }
      } finally {
        active = undefined;
      }
    });
    return active;
  };
}
