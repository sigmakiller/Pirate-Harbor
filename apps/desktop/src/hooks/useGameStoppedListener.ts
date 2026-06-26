/**
 * useGameStoppedListener — listens for `game-stopped` Tauri events.
 *
 * The Rust monitor (launcher.rs) emits `game-stopped` with the stopped
 * game's ID as payload when it detects the process has exited.
 *
 * Usage:
 *   useGameStoppedListener((gameId) => {
 *     // refresh data, show toast, etc.
 *   });
 *
 * The listener is automatically cleaned up on unmount.
 */

import { useEffect } from "react";
import { listen }   from "@tauri-apps/api/event";

type GameStoppedCallback = (gameId: string) => void;

/**
 * Registers a listener for the `game-stopped` Tauri event.
 *
 * @param onStopped - Called with the stopped game's ID each time the event fires.
 */
export function useGameStoppedListener(onStopped: GameStoppedCallback): void {
  useEffect(() => {
    // `listen` returns a Promise<UnlistenFn>. We store the unlisten fn
    // so we can call it on cleanup.
    let unlisten: (() => void) | undefined;

    listen<string>("game-stopped", (event) => {
      onStopped(event.payload);
    }).then((fn) => {
      unlisten = fn;
    });

    return () => {
      unlisten?.();
    };
  }, [onStopped]);
}
