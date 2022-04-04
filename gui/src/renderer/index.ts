import ReactDOM from 'react-dom';

import App from './app';

const app = new App();
const container = document.getElementById('app');
ReactDOM.render(app.renderView(), container);
