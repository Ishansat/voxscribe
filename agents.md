# GNET.md

This file provides development guidance, architectural specifications, and implementation patterns for **VoxScribe** when working with code in this repository.

---

## Project Overview

**VoxScribe** is a privacy-first, zero-API, local-first desktop meeting assistant built entirely from scratch. When an active video or audio meeting is detected on the device (via process monitoring and system audio loopback activity), VoxScribe automatically slides open a persistent, compact **Live Sidebar Panel**.

VoxScribe transcribes incoming meeting audio locally in real-time using **Whisper Large v3 Turbo**, and translates the speech into 200+ languages using a fully offline, local Rust neural machine translation engine (`trad`).

### UI Visual Representation

Inside the live sidebar stream, transcript output is displayed in distinct **blocks**:

* **Transcribed Text (Original Speech)**: Rendered in a **faded, subtle color** (`text-muted-foreground` / `opacity-50`).
* **Translated Text**: Rendered in a **normal, high-contrast text color** (`text-foreground`) for effortless readability.

### Key Technology Stack

* **Desktop Framework**: Tauri 2.x (Rust core + Next.js 14 / React 18 + Tailwind CSS)
* **Audio Capture & Mixing**: `cpal` (Microphone + System Audio Loopback mixing across macOS WASAPI/SCK/ALSA)
* **Voice Activity Detection (VAD)**: `silero-vad` / custom VAD engine for low-latency chunking
* **Speech-to-Text Engine**: `whisper-rs` configured with **Whisper Large v3 Turbo**
* **Offline Translation Engine**: `trad` crate (100% local, offline, CPU/GPU-optimized Rust NMT library supporting 200+ languages)
* **Meeting Presence Detector**: Audio threshold analyzer + active process detection (`sysinfo` process scanner for Zoom, Microsoft Teams, Google Meet, Slack, Webex)
* **IPC Surface**: Tauri commands and real-time event streaming (`app.emit`)

---

## Essential Development Commands

### Frontend & App Development

**Location**: `/` (Root repository)

```bash
# Dependency Installation
pnpm install                 # Install frontend and UI dependencies

# Development Servers
pnpm run dev                 # Next.js development server (port 3000)
pnpm run tauri:dev           # Full Tauri desktop application development mode (hot reload)
pnpm run tauri:build         # Production build and installer bundle generation

# GPU Hardware Acceleration Dev Modes
pnpm run tauri:dev:metal     # macOS Metal GPU acceleration (Apple Silicon / Intel Mac)
pnpm run tauri:dev:cuda      # NVIDIA CUDA GPU acceleration
pnpm run tauri:dev:vulkan    # AMD / Intel Vulkan GPU acceleration
pnpm run tauri:dev:cpu       # Standard CPU-only execution (Fallback)

```

---

### Cargo Dependencies (`src-tauri/Cargo.toml`)

```toml
[package]
name = "voxscribe"
version = "0.1.0"
edition = "2021"

[dependencies]
tauri = { version = "2.0", features = ["protocol-asset", "tray-icon"] }
whisper-rs = { version = "0.13", features = ["metal", "cuda"] }
trad = "1.0"               # Free, unlimited local translation engine (200+ languages)
cpal = "0.15"              # Cross-platform audio capture & loopback
sysinfo = "0.30"           # Meeting application process monitoring
tokio = { version = "1.0", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
uuid = { version = "1.6", features = ["v4"] }
chrono = { version = "0.4", features = ["serde"] }
anyhow = "1.0"

```

---

## High-Level Architecture

### Meeting Auto-Detection & Dual-Stream Sidebar Workflow

```
┌────────────────────────────────────────────────────────────────────────┐
│                        Background Audio Listener                       │
│              (Monitors Mic/System Audio & App Processes)               │
└──────────────────────────────────┬─────────────────────────────────────┘
                                   │
                      Meeting Detected (Auto-Trigger)
                                   ▼
┌────────────────────────────────────────────────────────────────────────┐
│                  VoxScribe Sidebar UI (Next.js)                         │
│  ┌──────────────────────────────────────────────────────────────────┐  │
│  │ Block Stream:                                                    │  │
│  │   • Transcribed Speech  ──► [ Faded / Muted Text Color ]       │  │
│  │   • Real-Time Translation ──► [ Normal / Prominent Color ]       │  │
│  └──────────────────────────────────────────────────────────────────┘  │
│                                  ▲ Tauri Events                        │
└──────────────────────────────────┼─────────────────────────────────────┘
                                   │
┌──────────────────────────────────┴─────────────────────────────────────┐
│                          Rust Core (Tauri)                             │
│ ┌────────────────┐    ┌────────────────────┐    ┌────────────────────┐ │
│ │ Audio Capture  │ ──►│ Whisper Engine     │ ──►│ Local Translation  │ │
│ │ (Mic + System) │    │ (Large v3 Turbo)   │    │ (`trad` Crate NMT) │ │
│ └────────────────┘    └────────────────────┘    └────────────────────┘ │
└────────────────────────────────────────────────────────────────────────┘

```

---

### Audio & Live Translation Pipeline

```
Raw Audio Input (Microphone + System Loopback Audio)
                         │
                         ▼
┌────────────────────────────────────────────────────────────┐
│                 Meeting Presence Detector                  │
│             (src-tauri/src/detector/meeting.rs)            │
└────────┬───────────────────────────────────────────────────┘
         │ (Meeting Status: Active)
         ▼
┌────────────────────────────────────────────────────────────┐
│                 Audio Pipeline Manager & VAD               │
│               (src-tauri/src/audio/pipeline.rs)            │
└────────────┬──────────────────────────────────┬────────────┘
             │                                  │
             ▼                                  ▼
   ┌──────────────────┐               ┌────────────────────┐
   │ Audio Recording  │               │ Live Speech Chunk  │
   │ (Pre-mixed WAV)  │               │ (VAD Segmented)    │
   └──────────────────┘               └─────────┬──────────┘
                                                │
                                                ▼
                                   WhisperEngine.transcribe()
                                     [Whisper Large v3 Turbo]
                                                │
                                                ├──► Transcribed Text (Faded Block)
                                                ▼
                                    Local Translator (`trad` Crate)
                                                │
                                                ├──► Translated Text (Normal Block)
                                                ▼
                                  Emit 'live-transcript-block'
                                                │
                                                ▼
                                    Sidebar UI Visual Render

```

---

### System & Module Directory Structure

```
voxscribe/
├── src/                            # Frontend UI (Next.js 14 + React 18)
│   ├── app/
│   │   ├── layout.tsx
│   │   └── page.tsx                # Sidebar entrypoint & overlay manager
│   ├── components/
│   │   └── Sidebar/
│   │       ├── LiveSidebar.tsx     # Sliding sidebar container component
│   │       ├── TranscriptBlock.tsx # Dual-color block display (Faded STT + Normal NMT)
│   │       └── Controls.tsx        # Language selector and auto-detect toggles
│   ├── hooks/
│   │   └── useMeetingDetector.ts   # Event hook for auto-open sidebar signals
│   └── styles/
│       └── globals.css
│
└── src-tauri/                      # Native Rust Backend
    ├── src/
    │   ├── audio/
    │   │   ├── devices.rs          # Input/output loopback device discovery
    │   │   ├── capture.rs          # cpal system audio + mic stream mixing
    │   │   ├── vad.rs              # Voice Activity Detection segmenter
    │   │   └── pipeline.rs         # Audio buffer pipeline dispatcher
    │   ├── detector/
    │   │   └── meeting.rs          # Process scanner (`sysinfo`) & signal analyzer
    │   ├── whisper/
    │   │   ├── engine.rs           # `whisper-rs` bindings for Large v3 Turbo
    │   │   └── model_loader.rs     # Local model downloader & weight verification
    │   ├── translation/
    │   │   └── engine.rs           # Offline translation via `trad` crate
    │   ├── commands.rs             # Tauri command invocations
    │   ├── lib.rs                  # App setup, tray config, and event orchestration
    │   └── main.rs                 # Tauri application entry point
    └── Cargo.toml

```

---

## Rust ↔ Sidebar Frontend Communication

### Command Pattern (Frontend → Rust)

```typescript
import { invoke } from '@tauri-apps/api/core';

// Manually start or re-configure live transcription & translation
await invoke('start_voxscribe_session', {
  micDevice: "Built-in Microphone",
  systemDevice: "System Loopback Audio",
  sourceLanguage: "auto", // Whisper auto-detection
  targetLanguage: "en"    // Destination translation language
});

```

---

### Event Pattern (Rust → Live Sidebar)

When speech is captured, transcribed by Whisper Large v3 Turbo, and translated by the `trad` engine, Rust emits the structured `live-transcript-block` payload:

```rust
// src-tauri/src/audio/pipeline.rs

#[derive(Clone, serde::Serialize)]
pub struct TranscriptBlockPayload {
    pub id: String,
    pub transcribed_text: String, // Original speech (Faded display in UI)
    pub translated_text: String,  // Translated text (Normal display in UI)
    pub source_lang: String,
    pub target_lang: String,
    pub timestamp: String,
}

// Emitting live block update to Next.js Frontend
app.emit("live-transcript-block", TranscriptBlockPayload {
    id: uuid::Uuid::new_v4().to_string(),
    transcribed_text: "こんにちは、ミーティングを始めましょう。".into(),
    translated_text: "Hello, let's start the meeting.".into(),
    source_lang: "ja".into(),
    target_lang: "en".into(),
    timestamp: chrono::Utc::now().to_rfc3339(),
})?;

```

---

### React Block Visual Component (`TranscriptBlock.tsx`)

This component implements the visual hierarchy: transcribed text in **faded/subtle style**, translated text in **normal/prominent style**:

```tsx
// src/components/Sidebar/TranscriptBlock.tsx
import React from 'react';

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
      {/* Transcribed Speech (Faded / Muted Visual) */}
      <p className="text-xs text-muted-foreground/60 dark:text-gray-400/50 mb-1.5 leading-relaxed font-normal select-text">
        {transcribedText}
      </p>

      {/* Real-time Translated Text (Normal / Prominent Visual) */}
      <p className="text-sm text-foreground font-semibold leading-normal select-text">
        {translatedText}
      </p>

      {/* Timestamp footer */}
      <div className="mt-1.5 flex justify-end items-center">
        <span className="text-[10px] text-muted-foreground/40 font-mono">
          {new Date(timestamp).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit', second: '2-digit' })}
        </span>
      </div>
    </div>
  );
};

```

---

## Free & Unlimited Local Translation Engine (`trad` Crate)

VoxScribe uses zero external paid translation APIs. All translation is executed locally on-device using the `trad` crate for 200+ offline languages:

```rust
// src-tauri/src/translation/engine.rs
use trad::Translator;

pub struct LocalTranslationEngine {
    translator: Translator,
}

impl LocalTranslationEngine {
    pub async fn new() -> anyhow::Result<Self> {
        // Initializes CPU/GPU-optimized local NMT translation model
        let translator = Translator::setup(None).await?;
        Ok(Self { translator })
    }

    pub async fn translate(&self, text: &str, src_lang: &str, target_lang: &str) -> anyhow::Result<String> {
        if text.trim().is_empty() {
            return Ok(String::new());
        }
        let translated = self.translator.translate(text, src_lang, target_lang).await?;
        Ok(translated)
    }
}

```

---

## Critical Development Patterns

1. **Automatic Meeting Detection**:
* A lightweight background thread monitors active desktop process handles (`sysinfo`) for known meeting apps (`Zoom.exe`, `Teams.exe`, `Slack.exe`, `Webex.exe`, `chrome.exe` Google Meet tabs).
* Concurrently, the system audio loopback RMS level is calculated. If sustained audio activity (>0.05 amplitude threshold) is detected alongside a meeting process, VoxScribe fires an `open-sidebar` event to slide out the UI panel.


2. **Dual-Stream Block Rendering**:
* Audio frames are processed in 1.5–3.0 second VAD windows.
* Once Whisper Large v3 Turbo outputs a transcribed clause, it is immediately passed to the local `trad` translation engine.
* The UI receives both strings in a single atomic event (`live-transcript-block`), preventing visual sync issues between transcription and translation.


3. **Performance Optimization**:
* Voice Activity Detection (VAD) drops silent audio segments before passing buffers to Whisper, reducing CPU/GPU load by up to 70%.
* Whisper Large v3 Turbo utilizes 8-bit or float16 quantization depending on target hardware capabilities (Metal / CUDA / CPU).



---

## Key Files Reference

* **Sidebar UI Layout**: `src/components/Sidebar/LiveSidebar.tsx`
* **Dual-Style Transcript Block**: `src/components/Sidebar/TranscriptBlock.tsx`
* **Meeting Auto-Detector**: `src-tauri/src/detector/meeting.rs`
* **Whisper Large v3 Turbo Engine**: `src-tauri/src/whisper/engine.rs`
* **Offline Translation Engine**: `src-tauri/src/translation/engine.rs`
* **Audio Capture & VAD Pipeline**: `src-tauri/src/audio/pipeline.rs`
