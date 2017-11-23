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

  it('shows secured hints when connected', () => {
    const component = renderConnected();

    const header = component.find(Header);
    const securityMessage = component.find('.connect__status-security--secure');
    const disconnectButton = component.find('.button .button--negative-light');

    expect(header.prop('style')).to.equal('success');
    expect(securityMessage.text().toLowerCase()).to.contain('secure');
    expect(disconnectButton.text()).to.equal('Disconnect');
  });

  it('shows the connection location when connecting', () => {
    const component = renderConnecting({
      getServerInfo: (_s) => ({
        address: '185.65.132.102',
        name: '',
        location: [0, 0],
        country: 'norway',
        city: 'oslo',
      }),
    }, {
      clientIp: '185.65.132.102',
    });
    const countryAndCity = component.find('.connect__status-location');
    const ipAddr = component.find('.connect__status-ipaddress');

    expect(countryAndCity.text()).to.contain('norway');
    expect(countryAndCity.text()).not.to.contain('oslo');
    expect(ipAddr.text()).to.be.empty;
  });

  it('shows the connection location when connected', () => {
    const component = renderConnected({
      getServerInfo: (_s) => ({
        address: '185.65.132.102',
        name: '',
        location: [0, 0],
        country: 'sweden',
        city: 'gothenburg',
      }),
    }, {
      clientIp: '185.65.132.102',
    });
    const countryAndCity = component.find('.connect__status-location');
    const ipAddr = component.find('.connect__status-ipaddress');

    expect(countryAndCity.text()).to.contain('sweden');
    expect(countryAndCity.text()).to.contain('gothenburg');
    expect(ipAddr.text()).to.contain('185.65.132.102');
  });

  it('shows the connection location when disconnected', () => {
    const component = renderNotConnected({
      getServerInfo: (_s) => ({
        address: '\u2003',
        name: '',
        location: [0, 0],
        country: 'sweden',
        city: 'gothenburg',
      }),
    }, {
      clientIp: '\u2003',
    });
    const countryAndCity = component.find('.connect__status-location');
    const ipAddr = component.find('.connect__status-ipaddress');

    expect(countryAndCity.text()).to.contain('\u2002');
    expect(countryAndCity.text()).to.not.contain('\u2003');
    expect(ipAddr.text()).to.contain('\u2003');
  });

  it('shows the country name in the location switcher', () => {
    const servers = {
      'se1.mullvad.net': { name: 'Sweden' },
    };
    const getServerInfo = (key) => servers[key] || defaultServer;
    const component = renderNotConnected({
      getServerInfo: getServerInfo,
    });
    const locationSwitcher = component.find('.connect__server');

    component.setProps({
      settings: {
        relaySettings: {
          host: 'se1.mullvad.net',
          protocol: 'udp',
          port: 1301,
        },
      },
    });
    expect(locationSwitcher.text()).to.contain(servers['se1.mullvad.net'].name);
  });

  it('invokes the onConnect prop', (done) => {
    const component = renderNotConnected({
      onConnect: () => done(),
    });
    const connectButton = component.find('.button .button--positive');

    connectButton.simulate('click');
  });
});

function renderNotConnected(customProps, customConnectionProps) {
  const connection = Object.assign({}, defaultConnection, {
    status: 'disconnected',
  }, customConnectionProps);

  const props = Object.assign({}, customProps, {connection});
  return renderWithProps(props);
}

function renderConnecting(customProps, customConnectionProps) {
  const connection = Object.assign({}, defaultConnection, {
    status: 'connecting',
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
  settings: {
    relaySettings: {
      host: 'www.example.com',
      protocol: 'udp',
      port: 1301,
    },
  },
  connection: defaultConnection,
};
