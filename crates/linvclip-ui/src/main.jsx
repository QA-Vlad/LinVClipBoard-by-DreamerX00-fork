import React from "react";
import ReactDOM from "react-dom/client";
import "@fontsource-variable/inter";
import { I18nProvider } from "./i18n/index.jsx";
import App from "./App";
import "./styles.css";

ReactDOM.createRoot(document.getElementById("root")).render(
  <React.StrictMode>
    <I18nProvider defaultLang="en">
      <App />
    </I18nProvider>
  </React.StrictMode>
);
