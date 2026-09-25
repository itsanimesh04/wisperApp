import React from "react";
import ReactDOM from "react-dom/client";
import SettingsApp from "./SettingsApp";
import OverlayApp from "./OverlayApp";
import "./index.css";

const params = new URLSearchParams(window.location.search);
const isOverlay = params.get("window") === "overlay";

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    {isOverlay ? <OverlayApp /> : <SettingsApp />}
  </React.StrictMode>
);
