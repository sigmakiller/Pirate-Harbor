/**
 * useEnrichmentProgress — Listen for metadata enrichment events.
 *
 * Subscribes to 'metadata-enrichment-progress' events emitted by the backend
 * during bulk enrichment operations.
 */

import { useEffect, useState } from "react";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

export interface EnrichmentProgress {
  total: number;
  completed: number;
  pending: number;
  failed: number;
}

export function useEnrichmentProgress() {
  const [progress, setProgress] = useState<EnrichmentProgress | null>(null);
  const [isActive, setIsActive] = useState(false);

  useEffect(() => {
    let unlisten: UnlistenFn | undefined;

    const setupListener = async () => {
      unlisten = await listen<EnrichmentProgress>(
        "metadata-enrichment-progress",
        (event) => {
          setProgress(event.payload);
          setIsActive(event.payload.pending > 0 || event.payload.completed < event.payload.total);
        }
      );
    };

    setupListener();

    return () => {
      if (unlisten) {
        unlisten();
      }
    };
  }, []);

  const reset = () => {
    setProgress(null);
    setIsActive(false);
  };

  return {
    progress,
    isActive,
    reset,
  };
}
