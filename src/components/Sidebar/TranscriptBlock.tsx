import React from "react";

export interface TranscriptBlockProps {
  id: string;
  transcribedText: string;
  translatedText: string;
  sourceLang?: string;
  targetLang?: string;
  timestamp: string;
}

export const TranscriptBlock: React.FC<TranscriptBlockProps> = ({
  transcribedText,
  translatedText,
  timestamp,
}) => {
  return (
    <div className="mb-3 p-3 rounded-lg bg-card/80 border border-border/60 shadow-sm transition-all hover:border-border">
      <p className="text-xs text-muted-foreground/60 dark:text-gray-400/50 mb-1.5 leading-relaxed font-normal select-text">
        {transcribedText}
      </p>

      <p className="text-sm text-foreground font-semibold leading-normal select-text">
        {translatedText}
      </p>

      <div className="mt-1.5 flex justify-end items-center">
        <span className="text-[10px] text-muted-foreground/40 font-mono">
          {new Date(timestamp).toLocaleTimeString([], { hour: "2-digit", minute: "2-digit", second: "2-digit" })}
        </span>
      </div>
    </div>
  );
};
