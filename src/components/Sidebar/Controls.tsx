"use client";

import React, { useState, useRef, useEffect } from "react";

export const LANGUAGES = [
  { code: "en", label: "English" },
  { code: "es", label: "Spanish" },
  { code: "fr", label: "French" },
  { code: "de", label: "German" },
  { code: "it", label: "Italian" },
  { code: "pt", label: "Portuguese" },
  { code: "ru", label: "Russian" },
  { code: "ja", label: "Japanese" },
  { code: "ko", label: "Korean" },
  { code: "zh", label: "Chinese (Simplified)" },
  { code: "ar", label: "Arabic" },
  { code: "hi", label: "Hindi" },
  { code: "nl", label: "Dutch" },
  { code: "pl", label: "Polish" },
  { code: "tr", label: "Turkish" },
  { code: "vi", label: "Vietnamese" },
  { code: "th", label: "Thai" },
  { code: "sv", label: "Swedish" },
  { code: "da", label: "Danish" },
  { code: "fi", label: "Finnish" },
] as const;

interface ControlsProps {
  targetLanguage: string;
  onTargetLanguageChange: (code: string) => void;
  autoDetect: boolean;
  onAutoDetectChange: (enabled: boolean) => void;
}

export const Controls: React.FC<ControlsProps> = ({
  targetLanguage,
  onTargetLanguageChange,
  autoDetect,
  onAutoDetectChange,
}) => {
  const [open, setOpen] = useState(false);
  const ref = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const handler = (e: MouseEvent) => {
      if (ref.current && !ref.current.contains(e.target as Node)) {
        setOpen(false);
      }
    };
    document.addEventListener("mousedown", handler);
    return () => document.removeEventListener("mousedown", handler);
  }, []);

  const selected = LANGUAGES.find((l) => l.code === targetLanguage);

  return (
    <div className="flex items-center gap-3 px-4 py-2.5 border-b border-border bg-muted/30">
      <div ref={ref} className="relative">
        <button
          type="button"
          onClick={() => setOpen(!open)}
          className="flex items-center gap-1.5 px-2.5 py-1.5 text-xs text-foreground bg-card border border-border rounded-md hover:bg-muted/50 transition-colors"
        >
          {selected?.label ?? "English"}
          <svg
            className={`w-3 h-3 text-muted-foreground transition-transform ${open ? "rotate-180" : ""}`}
            fill="none"
            viewBox="0 0 24 24"
            stroke="currentColor"
          >
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 9l-7 7-7-7" />
          </svg>
        </button>

        {open && (
          <div className="absolute top-full left-0 mt-1 z-50 w-44 max-h-56 overflow-y-auto bg-card border border-border rounded-lg shadow-xl">
            {LANGUAGES.map((lang) => (
              <button
                key={lang.code}
                type="button"
                onClick={() => {
                  onTargetLanguageChange(lang.code);
                  setOpen(false);
                }}
                className={`w-full text-left px-3 py-1.5 text-xs transition-colors ${
                  lang.code === targetLanguage
                    ? "text-foreground font-semibold bg-muted/50"
                    : "text-muted-foreground hover:text-foreground hover:bg-muted/30"
                }`}
              >
                {lang.label}
              </button>
            ))}
          </div>
        )}
      </div>

      <label className="flex items-center gap-2 cursor-pointer">
        <span className="text-xs text-muted-foreground select-none">Auto</span>
        <button
          type="button"
          role="switch"
          aria-checked={autoDetect}
          onClick={() => onAutoDetectChange(!autoDetect)}
          className={`relative inline-flex h-4 w-7 items-center rounded-full transition-colors ${
            autoDetect ? "bg-foreground" : "bg-muted-foreground/30"
          }`}
        >
          <span
            className={`inline-block h-3 w-3 rounded-full bg-card shadow-sm transition-transform ${
              autoDetect ? "translate-x-3.5" : "translate-x-0.5"
            }`}
          />
        </button>
      </label>
    </div>
  );
};
