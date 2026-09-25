import { useState, useEffect } from "react";
import { listen } from "@tauri-apps/api/event";
import "./overlay.css";

type OverlayState = "idle" | "recording" | "processing" | "done" | "error";

export default function OverlayApp() {
  const [state, setState] = useState<OverlayState>("idle");
  const [errorMsg, setErrorMsg] = useState("");

  useEffect(() => {
    const unlisten: (() => void)[] = [];

    listen("recording-started", () => {
      setState("recording");
      setErrorMsg("");
    }).then((fn) => unlisten.push(fn));

    listen("transcription-started", () => {
      setState("processing");
    }).then((fn) => unlisten.push(fn));

    listen("transcription-complete", () => {
      setState("done");
      setTimeout(() => setState("idle"), 1200);
    }).then((fn) => unlisten.push(fn));

    listen<string>("transcription-error", (event) => {
      setState("error");
      setErrorMsg(String(event.payload).slice(0, 60));
      setTimeout(() => setState("idle"), 3000);
    }).then((fn) => unlisten.push(fn));

    return () => {
      unlisten.forEach((fn) => fn());
    };
  }, []);

  if (state === "idle") {
    return <div className="overlay-root transparent" />;
  }

  return (
    <div className={`overlay-root ${state}`}>
      <div className="overlay-pill">
        {state === "recording" && (
          <>
            <span className="pulse-dot" />
            <span className="overlay-text">Recording…</span>
          </>
        )}
        {state === "processing" && (
          <>
            <span className="spinner" />
            <span className="overlay-text">Processing…</span>
          </>
        )}
        {state === "done" && (
          <>
            <span className="check-icon">✓</span>
            <span className="overlay-text">Done</span>
          </>
        )}
        {state === "error" && (
          <>
            <span className="error-icon">✕</span>
            <span className="overlay-text">{errorMsg || "Error"}</span>
          </>
        )}
      </div>
    </div>
  );
}
