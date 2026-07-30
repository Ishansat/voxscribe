"use client";

import React, { useState, useEffect, useRef, useCallback } from "react";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { TranscriptBlock } from "./TranscriptBlock";
import type { TranscriptBlockProps } from "./TranscriptBlock";
import { Controls } from "./Controls";

interface TranscriptBlockPayload {
  id: string;
  transcribed_text: string;
  translated_text: string;
  source_lang: string;
  target_lang: string;
  timestamp: string;
}

function mapPayload(p: TranscriptBlockPayload): TranscriptBlockProps {
  return {
    id: p.id,
    transcribedText: p.transcribed_text,
    translatedText: p.translated_text,
    sourceLang: p.source_lang,
    targetLang: p.target_lang,
    timestamp: p.timestamp,
  };
}

export const LiveSidebar: React.FC = () => {
  const [blocks, setBlocks] = useState<TranscriptBlockProps[]>([]);
  const [isVisible, setIsVisible] = useState(false);
  const [targetLanguage, setTargetLanguage] = useState("en");
  const [autoDetect, setAutoDetect] = useState(true);
  const scrollRef = useRef<HTMLDivElement>(null);
  const seenIds = useRef(new Set<string>());

  const scrollToBottom = useCallback(() => {
    requestAnimationFrame(() => {
      if (scrollRef.current) {
        scrollRef.current.scrollTop = scrollRef.current.scrollHeight;
      }
    });
  }, []);

  useEffect(() => {
    const unlisteners: UnlistenFn[] = [];
    let cancelled = false;

    async function setup() {
      try {
        const u1 = await listen<TranscriptBlockPayload>("live-transcript-block", (event) => {
          if (cancelled) return;
          const { id } = event.payload;
          if (seenIds.current.has(id)) return;
          seenIds.current.add(id);
          setBlocks((prev) => [...prev, mapPayload(event.payload)]);
          scrollToBottom();
        });
        unlisteners.push(u1);
      } catch {
        /* not in Tauri runtime */
      }

      try {
        const u2 = await listen("open-sidebar", () => {
          if (!cancelled) setIsVisible(true);
        });
        unlisteners.push(u2);
      } catch {
        /* not in Tauri runtime */
      }

      try {
        const u3 = await listen("close-sidebar", () => {
          if (!cancelled) setIsVisible(false);
        });
        unlisteners.push(u3);
      } catch {
        /* not in Tauri runtime */
      }
    }

    setup();

    return () => {
      cancelled = true;
      unlisteners.forEach((u) => u());
    };
  }, [scrollToBottom]);

  return (
    <div
      className={`fixed right-0 top-0 h-full w-[380px] bg-background border-l border-border shadow-2xl transition-transform duration-300 ease-out z-50 ${
        isVisible ? "translate-x-0" : "translate-x-full"
      }`}
    >
      <div className="flex flex-col h-full">
        <div className="flex items-center justify-between px-4 py-3 border-b border-border shrink-0">
          <h1 className="text-base font-semibold text-foreground tracking-tight">
            VoxScribe
          </h1>
          <button
            type="button"
            onClick={() => setIsVisible(false)}
            className="p-1 rounded-md text-muted-foreground hover:text-foreground hover:bg-muted/50 transition-colors"
            aria-label="Close sidebar"
          >
            <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>

        <Controls
          targetLanguage={targetLanguage}
          onTargetLanguageChange={setTargetLanguage}
          autoDetect={autoDetect}
          onAutoDetectChange={setAutoDetect}
        />

        <div ref={scrollRef} className="flex-1 overflow-y-auto p-3 space-y-1.5 scroll-smooth">
          {blocks.length === 0 && (
            <div className="flex items-center justify-center h-full">
              <p className="text-xs text-muted-foreground/40 select-none">
                Waiting for speech&hellip;
              </p>
            </div>
          )}
          {blocks.map((block) => (
            <TranscriptBlock key={block.id} {...block} />
          ))}
        </div>

        <div className="shrink-0 px-4 py-2 border-t border-border flex items-center justify-between">
          <span className="text-[10px] text-muted-foreground/40">
            {blocks.length} {blocks.length === 1 ? "block" : "blocks"}
          </span>
          <button
            type="button"
            onClick={() => {
              setBlocks([]);
              seenIds.current.clear();
            }}
            className="text-[10px] text-muted-foreground/40 hover:text-muted-foreground transition-colors"
          >
            Clear
          </button>
        </div>
      </div>
    </div>
  );
};
