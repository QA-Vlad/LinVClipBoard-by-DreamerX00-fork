import React from "react";
import ReactDOM from "react-dom/client";
import "@fontsource-variable/inter";
import { I18nProvider } from "./i18n/index.jsx";
import { KeybindingProvider } from "./contexts/KeybindingContext.jsx";
import ErrorBoundary from "./components/ErrorBoundary.jsx";
import App from "./App";
import "./styles.css";

ReactDOM.createRoot(document.getElementById("root")).render(
  <React.StrictMode>
    <ErrorBoundary>
      <I18nProvider defaultLang="en">
        <KeybindingProvider>
          <App />
        </KeybindingProvider>
      </I18nProvider>
    </ErrorBoundary>
  </React.StrictMode>
);
