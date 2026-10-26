import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';
import '@fontsource/inter/latin-300.css';
import '@fontsource/inter/latin-400.css';
import '@fontsource/inter/latin-500.css';
import '@fontsource/inter/latin-600.css';
import '@fontsource/inter/latin-700.css';
import App from './App';
import './styles.css';

const raiz = document.getElementById('root');
if (!raiz) {
  throw new Error('falta #root');
}

createRoot(raiz).render(
  <StrictMode>
    <App />
  </StrictMode>,
);
