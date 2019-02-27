import * as RX from 'reactxp';
import App from './app';

const app = new App();
const view = app.renderView();

RX.App.initialize(true, true);
RX.UserInterface.setMainView(view);
RX.UserInterface.useCustomScrollbars(true);
