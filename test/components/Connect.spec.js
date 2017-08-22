// @flow

import { expect } from 'chai';
import React from 'react';
import { mount } from 'enzyme';

import Connect from '../../app/components/Connect';
import Header from '../../app/components/HeaderBar';

import type { ReactWrapper } from 'enzyme';

describe('components/Connect', () => {

  it('shows unsecured hints when not connected', () => {
    const component = renderNotConnected();

    const header = component.find(Header);
    const securityMessage = component.find('.connect__status-security--unsecured');
    const connectButton = component.find('.button .button--positive');

    expect(header.prop('style')).to.equal('error');
    expect(securityMessage.text().toLowerCase()).to.contain('unsecured');
    expect(connectButton.text()).to.equal('Secure my connection');
  });

  it('invokes the onConnect prop', (done) => {
    const component = renderNotConnected({
      onConnect: done,
    });
    const connectButton = component.find('.button .button--positive');

    connectButton.simulate('click');
  });

  it('shows the connection location information when connected', () => {
    const component = renderConnected({}, {
      country: 'sweden',
      city: 'gothenburg',
      clientIp: '1.2.3.4',
    });
    const countryAndCity = component.find('.connect__status-location');
    const ipAddr = component.find('.connect__status-ipaddress');

    expect(countryAndCity.text()).to.contain('sweden');
    expect(countryAndCity.text()).to.contain('gothenburg');
    expect(ipAddr.text()).to.contain('1.2.3.4');
  });
});

function renderNotConnected(customProps, customConnectionProps) {
  const connection = Object.assign({}, defaultConnection, {
    status: 'disconnected',
  }, customConnectionProps);

  const props = Object.assign({}, customProps, {connection});
  return renderWithProps(props);
}

function renderConnected(customProps, customConnectionProps) {
  const connection = Object.assign({}, defaultConnection, {
    status: 'connected',
  }, customConnectionProps);

  const props = Object.assign({}, customProps, {connection});
  return renderWithProps(props);
}

function renderWithProps(customProps): ReactWrapper {
  const props = Object.assign({}, defaultProps, customProps);
  return mount( <Connect { ...props } /> );
}

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
