"use client";

import { useState, useEffect } from "react";

export function useMeetingDetector() {
  const [isMeetingActive, setIsMeetingActive] = useState(false);

  useEffect(() => {
    setIsMeetingActive(false);
  }, []);

  return { isMeetingActive };
}
