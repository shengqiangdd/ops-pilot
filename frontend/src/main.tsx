import React from 'react';
import ReactDOM from 'react-dom/client';
import { BrowserRouter } from 'react-router-dom';
import { App } from './App';
import { ToastProvider } from './components/Toast';
import { ThemeProvider } from './components/ThemeProvider';
import { I18nProvider } from './i18n';
import { QueryProvider } from './lib/queryClient.tsx';
import './index.css';

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <BrowserRouter>
      <QueryProvider>
        <I18nProvider>
          <ThemeProvider>
            <ToastProvider>
              <App />
            </ToastProvider>
          </ThemeProvider>
        </I18nProvider>
      </QueryProvider>
    </BrowserRouter>
  </React.StrictMode>,
);
