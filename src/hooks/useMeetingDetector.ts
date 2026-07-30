"use client";

import { useState, useEffect } from "react";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

export function useMeetingDetector() {
  const [isMeetingActive, setIsMeetingActive] = useState(false);

  useEffect(() => {
    const unlisteners: UnlistenFn[] = [];

    async function setup() {
      try {
        const u1 = await listen("open-sidebar", () => setIsMeetingActive(true));
        unlisteners.push(u1);
      } catch {
        /* not in Tauri runtime */
      }

      try {
        const u2 = await listen("close-sidebar", () => setIsMeetingActive(false));
        unlisteners.push(u2);
      } catch {
        /* not in Tauri runtime */
      }
    }

    setup();

    return () => {
      unlisteners.forEach((u) => u());
    };
  }, []);

  return { isMeetingActive };
}
