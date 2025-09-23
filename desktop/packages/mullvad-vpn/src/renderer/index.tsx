import { createRoot } from 'react-dom/client';

import App from './app';
// import { TestApp } from './test-app';

const app = new App();
const container = document.getElementById('app');
const root = createRoot(container!);
root.render(app.renderView());
// root.render(<TestApp />);
