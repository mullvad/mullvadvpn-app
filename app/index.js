// @flow

import RX from 'reactxp';
import App from './app';

const app = new App();
const view = app.renderView();

app.connect();

RX.App.initialize(true, true);
RX.UserInterface.setMainView(view);
RX.UserInterface.useCustomScrollbars(true);
