// @flow

import { expect } from 'chai';
import React from 'react';
import { mount } from 'enzyme';

import Connect from '../../app/components/Connect';
import Header from '../../app/components/HeaderBar';

describe('components/Connect', () => {

  it('shows unsecured hints when not connected', () => {
    const component = mount( <Connect {...defaultProps} /> );

    const header = component.find(Header);
    const securityMessage = component.find('.connect__status-security--unsecured');
    const connectButton = component.find('.button .button--positive');

    expect(header.prop('style')).to.equal('error');
    expect(securityMessage.text().toLowerCase()).to.contain('unsecured');
    expect(connectButton.text()).to.equal('Secure my connection');
  });
});

const noop = () => {};
const defaultServer = {
  address: '',
  name: '',
  city: '',
  country: '',
  location: [0, 0],
};
const defaultConnection = {
  status: 'disconnected',
  isOnline: true,
  serverAddress: null,
  clientIp: null,
  location: null,
  country: null,
  city: null,
};

const defaultProps = {
  onSettings: noop,
  onSelectLocation: noop,
  onConnect: noop,
  onCopyIP: noop,
  onDisconnect: noop,
  onExternalLink: noop,
  getServerInfo: (_) => { return defaultServer; },

  accountExpiry: '',
  preferredServer: '',
  connection: defaultConnection,
};
