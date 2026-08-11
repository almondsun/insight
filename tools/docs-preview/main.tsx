import React from "react";
import ReactDOM from "react-dom/client";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import App from "../../src/App";
import { docsApi } from "./fixtures";
import "../../src/styles.css";
import "../../src/logo.css";
import "../../src/theme.css";
import "../../src/pagination.css";
import "../../src/product.css";
import "../../src/onboarding.css";
import "../../src/smart-lists.css";
import "../../src/import-warnings.css";

const queryClient=new QueryClient({defaultOptions:{queries:{staleTime:Infinity,retry:false},mutations:{retry:false}}});
ReactDOM.createRoot(document.getElementById("root")!).render(<React.StrictMode><QueryClientProvider client={queryClient}><App client={docsApi}/></QueryClientProvider></React.StrictMode>);
