import React from 'react';
import RX from 'reactxp';
import App from './app';

RX.App.initialize(true, true);
RX.UserInterface.setMainView(<App />);
RX.UserInterface.useCustomScrollbars(true);
