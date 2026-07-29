"use client";

import React, { useState, useEffect, useCallback } from "react";
import { TranscriptBlock } from "./TranscriptBlock";
import type { TranscriptBlockProps } from "./TranscriptBlock";
import { Controls } from "./Controls";

export const LiveSidebar: React.FC = () => {
  const [blocks, setBlocks] = useState<TranscriptBlockProps[]>([]);
  const [isVisible, setIsVisible] = useState(false);

  useEffect(() => {
    const unlisten = window.__TAURI_INTERNALS__?.invoke
      ? null
      : null;

    return () => {
      if (unlisten) unlisten();
    };
  }, []);

  return (
    <div
      className={`fixed right-0 top-0 h-full w-[380px] bg-background border-l border-border shadow-2xl transition-transform duration-300 ${
        isVisible ? "translate-x-0" : "translate-x-full"
      }`}
    >
      <div className="flex flex-col h-full">
        <div className="flex items-center justify-between p-4 border-b border-border">
          <h1 className="text-lg font-semibold text-foreground">VoxScribe</h1>
        </div>

        <Controls />

        <div className="flex-1 overflow-y-auto p-4 space-y-2">
          {blocks.map((block) => (
            <TranscriptBlock key={block.id} {...block} />
          ))}
        </div>
      </div>
    </div>
  );
};
