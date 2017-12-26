// @flow

import { expect } from 'chai';
import React from 'react';
import { mount } from 'enzyme';

import Connect from '../../app/components/Connect';
import Header from '../../app/components/HeaderBar';

import type { ConnectProps } from '../../app/components/Connect';

describe('components/Connect', () => {

  it('shows unsecured hints when disconnected', () => {
    const component = renderWithProps({
      connection: {
        ...defaultProps.connection,
        status: 'disconnected',
      }
    });

    const header = component.find(Header);
    const securityMessage = component.find('.connect__status-security--unsecured');
    const connectButton = component.find('.button .button--positive');

    expect(header.prop('style')).to.equal('error');
    expect(securityMessage.text().toLowerCase()).to.contain('unsecured');
    expect(connectButton.text()).to.equal('Secure my connection');
  });

  it('shows secured hints when connected', () => {
    const component = renderWithProps({
      connection: {
        ...defaultProps.connection,
        status: 'connected',
      }
    });

    const header = component.find(Header);
    const securityMessage = component.find('.connect__status-security--secure');
    const disconnectButton = component.find('.button .button--negative-light');

    expect(header.prop('style')).to.equal('success');
    expect(securityMessage.text().toLowerCase()).to.contain('secure');
    expect(disconnectButton.text()).to.equal('Disconnect');
  });

  it('shows the connection location when connecting', () => {
    const component = renderWithProps({
      connection: {
        ...defaultProps.connection,
        status: 'connecting',
        country: 'Norway',
        city: 'Oslo',
      }
    });
    const countryAndCity = component.find('.connect__status-location');
    const ipAddr = component.find('.connect__status-ipaddress');

    expect(countryAndCity.text()).to.contain('Norway');
    expect(countryAndCity.text()).not.to.contain('Oslo');
    expect(ipAddr.text()).to.be.empty;
  });

  it('shows the connection location when connected', () => {
    const component = renderWithProps({
      connection: {
        ...defaultProps.connection,
        status: 'connected',
        country: 'Norway',
        city: 'Oslo',
        clientIp: '4.3.2.1',
      }
    });
    const countryAndCity = component.find('.connect__status-location');
    const ipAddr = component.find('.connect__status-ipaddress');

    expect(countryAndCity.text()).to.contain('Norway');
    expect(countryAndCity.text()).to.contain('Oslo');
    expect(ipAddr.text()).to.contain('4.3.2.1');
  });

  it('shows the connection location when disconnected', () => {
    const component = renderWithProps({
      connection: {
        ...defaultProps.connection,
        status: 'disconnected',
        country: 'Norway',
        city: 'Oslo',
        clientIp: '4.3.2.1',
      }
    });
    const countryAndCity = component.find('.connect__status-location');
    const ipAddr = component.find('.connect__status-ipaddress');

    expect(countryAndCity.text()).to.contain('\u2002');
    expect(countryAndCity.text()).to.not.contain('Oslo');
    expect(ipAddr.text()).to.contain('\u2003');
  });

  it('shows the country name in the location switcher', () => {
    const component = renderWithProps({
      connection: {
        ...defaultProps.connection,
        status: 'disconnected',
      },
      settings: {
        ...defaultProps.settings,
        relaySettings: {
          normal: {
            location: { city: ['se', 'mma'] },
            protocol: 'any',
            port: 'any',
          }
        },
      },
    });

    const locationSwitcher = component.find('.connect__server');
    expect(locationSwitcher.text()).to.contain('Malmö');
  });

  it('invokes the onConnect prop', (done) => {
    const component = renderWithProps({
      onConnect: () => done(),
      connection: {
        ...defaultProps.connection,
        status: 'disconnected',
      }
    });
    const connectButton = component.find('.button .button--positive');

    connectButton.simulate('click');
  });
});

const defaultProps: ConnectProps = {
  onSettings: () => {},
  onSelectLocation: () => {},
  onConnect: () => {},
  onCopyIP: () => {},
  onDisconnect: () => {},
  onExternalLink: () => {},
  accountExpiry: '',
  settings: {
    relaySettings: {
      normal: {
        location: 'any',
        protocol: 'any',
        port: 'any',
      }
    },
    relayLocations: [{
      name: 'Sweden',
      code: 'se',
      hasActiveRelays: true,
      cities: [{
        name: 'Malmö',
        code: 'mma',
        hasActiveRelays: true,
        position: [0, 0],
      }]
    }],
    allowLan: false,
  },
  connection: {
    status: 'disconnected',
    isOnline: true,
    clientIp: null,
    location: null,
    country: null,
    city: null,
  },
};

function renderWithProps(customProps: $Shape<ConnectProps>) {
  const props = { ...defaultProps, ...customProps };
  return mount( <Connect { ...props } /> );
}
