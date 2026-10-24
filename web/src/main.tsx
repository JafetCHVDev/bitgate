import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { App } from "./App";
import { ChainProvider, LangProvider, WalletProvider } from "./ui/state";
import "./styles.css";

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <LangProvider>
      <ChainProvider>
        <WalletProvider>
          <App />
        </WalletProvider>
      </ChainProvider>
    </LangProvider>
  </StrictMode>,
);
