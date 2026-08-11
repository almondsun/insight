import React from "react";
import ReactDOM from "react-dom/client";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import App from "./App";
import "./styles.css";
import "./logo.css";
import "./theme.css";
import "./pagination.css";
import "./product.css";
import "./onboarding.css";
import "./smart-lists.css";
import "./import-warnings.css";

const queryClient = new QueryClient({defaultOptions:{queries:{staleTime:10_000,retry:false}}});
ReactDOM.createRoot(document.getElementById("root")!).render(<React.StrictMode><QueryClientProvider client={queryClient}><App /></QueryClientProvider></React.StrictMode>);
