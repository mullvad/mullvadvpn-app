// @flow

import * as React from 'react';
import { shallow } from 'enzyme';

import Connect from '../../src/renderer/components/Connect';

type ConnectProps = React.ElementProps<typeof Connect>;

describe('components/Connect', () => {
  it('shows unsecured hints when disconnected', () => {
    const component = renderWithProps({
      connection: {
        ...defaultProps.connection,
        status: 'disconnected',
      },
    });

    const header = getComponent(component, 'header');
    const securityMessage = getComponent(component, 'networkSecurityMessage');
    const connectButton = getComponent(component, 'secureConnection');
    expect(header.prop('barStyle')).to.equal('error');
    expect(securityMessage.html()).to.contain('UNSECURED');
    expect(connectButton.html()).to.contain('Secure my connection');
  });

  it('shows secured hints when connected', () => {
    const component = renderWithProps({
      connection: {
        ...defaultProps.connection,
        status: 'connected',
      },
    });

    const header = getComponent(component, 'header');
    const securityMessage = getComponent(component, 'networkSecurityMessage');
    const disconnectButton = getComponent(component, 'disconnect');
    expect(header.prop('barStyle')).to.equal('success');
    expect(securityMessage.html()).to.contain('SECURE ');
    expect(disconnectButton.html()).to.contain('Disconnect');
  });

  it('shows the connection location when connecting', () => {
    const component = renderWithProps({
      connection: {
        ...defaultProps.connection,
        status: 'connecting',
        country: 'Norway',
        city: 'Oslo',
      },
    });
    const countryAndCity = getComponent(component, 'location');
    const ipAddr = getComponent(component, 'ipAddress');

    expect(countryAndCity.html()).to.contain('Norway');
    expect(countryAndCity.html()).not.to.contain('Oslo');
    expect(ipAddr.length).to.equal(0);
  });

  it('shows the connection location when connected', () => {
    const component = renderWithProps({
      connection: {
        ...defaultProps.connection,
        status: 'connected',
        country: 'Norway',
        city: 'Oslo',
        ip: '4.3.2.1',
      },
    });
    const countryAndCity = getComponent(component, 'location');
    const ipAddr = getComponent(component, 'ipAddress');

    expect(countryAndCity.html()).to.contain('Norway');
    expect(countryAndCity.html()).to.contain('Oslo');
    expect(ipAddr.html()).to.contain('4.3.2.1');
  });

  it('shows the connection location when disconnected', () => {
    const component = renderWithProps({
      connection: {
        ...defaultProps.connection,
        status: 'disconnected',
        country: 'Norway',
        city: 'Oslo',
        ip: '4.3.2.1',
      },
    });
    const countryAndCity = getComponent(component, 'location');
    const ipAddr = getComponent(component, 'ipAddress');

    expect(countryAndCity.html()).to.contain('Norway');
    expect(countryAndCity.html()).to.not.contain('Oslo');
    expect(ipAddr.html()).to.contain('4.3.2.1');
  });

  it('invokes the onConnect prop', (done) => {
    const component = renderWithProps({
      onConnect: () => done(),
      connection: {
        ...defaultProps.connection,
        status: 'disconnected',
      },
    });
    const connectButton = getComponent(component, 'secureConnection');

    connectButton.prop('onPress')();
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
  selectedRelayName: '',
  connection: {
    status: 'disconnected',
    isOnline: true,
    ip: null,
    latitude: null,
    longitude: null,
    country: null,
    city: null,
  },
  updateAccountExpiry: () => Promise.resolve(),
};

function renderWithProps(customProps: $Shape<ConnectProps>) {
  const props = { ...defaultProps, ...customProps };
  return shallow(<Connect {...props} />);
}

function getComponent(container, testName) {
  return container.findWhere((n) => n.prop('testName') === testName);
}
