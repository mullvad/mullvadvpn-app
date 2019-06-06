import moment from 'moment';
import * as React from 'react';
import { shallow } from 'enzyme';
import NotificationArea from '../../src/renderer/components/NotificationArea';
import { AfterDisconnect } from '../../src/shared/daemon-rpc-types';
import AccountExpiry from '../../src/renderer/lib/account-expiry';
import { expect } from 'chai';

describe('components/NotificationArea', () => {
  const defaultVersion = {
    consistent: true,
    currentIsSupported: true,
    upToDate: true,
    current: '2018.2',
    latest: '2018.2-beta1',
    latestStable: '2018.2',
  };

  const defaultExpiry = new AccountExpiry(
    moment()
      .add(1, 'year')
      .format(),
    'en',
  );

  it('handles disconnecting state', () => {
    for (const reason of ['nothing', 'block'] as AfterDisconnect[]) {
      const component = shallow(
        <NotificationArea
          tunnelState={{
            state: 'disconnecting',
            details: reason,
          }}
          version={defaultVersion}
          accountExpiry={defaultExpiry}
          openExternalLink={() => {}}
          blockWhenDisconnected={false}
        />,
      );
      expect(component.state('visible')).to.be.false;
    }
  });

  it('handles disconnecting state when reconnecting', () => {
    const component = shallow(
      <NotificationArea
        tunnelState={{
          state: 'disconnecting',
          details: 'reconnect',
        }}
        version={defaultVersion}
        accountExpiry={defaultExpiry}
        openExternalLink={() => {}}
        blockWhenDisconnected={false}
      />,
    );
    expect(component.state('visible')).to.be.true;
  });

  it('handles connected state', () => {
    const component = shallow(
      <NotificationArea
        tunnelState={{
          state: 'connected',
          details: {
            address: '1.2.3.4',
            protocol: 'tcp',
            tunnelType: 'openvpn',
          },
        }}
        version={defaultVersion}
        accountExpiry={defaultExpiry}
        openExternalLink={() => {}}
        blockWhenDisconnected={false}
      />,
    );

    expect(component.state('visible')).to.be.false;
  });

  it('handles disconnected state', () => {
    const component = shallow(
      <NotificationArea
        tunnelState={{
          state: 'disconnected',
        }}
        version={defaultVersion}
        accountExpiry={defaultExpiry}
        openExternalLink={() => {}}
        blockWhenDisconnected={false}
      />,
    );

    expect(component.state('visible')).to.be.false;
  });

  it('handles disconnected state, blocking when connected', () => {
    const component = shallow(
      <NotificationArea
        tunnelState={{
          state: 'disconnected',
        }}
        version={defaultVersion}
        accountExpiry={defaultExpiry}
        openExternalLink={() => {}}
        blockWhenDisconnected={true}
      />,
    );

    expect(component.state('visible')).to.be.true;
  });

  it('handles connecting state', () => {
    const component = shallow(
      <NotificationArea
        tunnelState={{
          state: 'connecting',
        }}
        version={defaultVersion}
        accountExpiry={defaultExpiry}
        openExternalLink={() => {}}
        blockWhenDisconnected={false}
      />,
    );

    expect(component.state('type')).to.be.equal('blocking');
    expect(component.state('visible')).to.be.true;
  });

  it('handles blocked state', () => {
    const component = shallow(
      <NotificationArea
        tunnelState={{
          state: 'blocked',
          details: {
            reason: 'no_matching_relay',
          },
        }}
        version={defaultVersion}
        accountExpiry={defaultExpiry}
        openExternalLink={() => {}}
        blockWhenDisconnected={false}
      />,
    );

    expect(component.state('type')).to.be.equal('blocking');
    expect(component.state('visible')).to.be.true;
  });

  it('handles inconsistent version', () => {
    const component = shallow(
      <NotificationArea
        tunnelState={{
          state: 'disconnected',
        }}
        version={{
          ...defaultVersion,
          consistent: false,
        }}
        accountExpiry={defaultExpiry}
        openExternalLink={() => {}}
        blockWhenDisconnected={false}
      />,
    );

    expect(component.state('type')).to.be.equal('inconsistent-version');
    expect(component.state('visible')).to.be.true;
  });

  it('handles unsupported version', () => {
    const component = shallow(
      <NotificationArea
        tunnelState={{
          state: 'disconnected',
        }}
        version={{
          ...defaultVersion,
          currentIsSupported: false,
          upToDate: false,
          current: '2018.1',
          nextUpgrade: '2018.2',
        }}
        accountExpiry={defaultExpiry}
        openExternalLink={() => {}}
        blockWhenDisconnected={false}
      />,
    );

    expect(component.state('type')).to.be.equal('unsupported-version');
    expect(component.state('visible')).to.be.true;
  });

  it('handles stable update available', () => {
    const component = shallow(
      <NotificationArea
        tunnelState={{
          state: 'disconnected',
        }}
        version={{
          ...defaultVersion,
          upToDate: false,
          current: '2018.2',
          latest: '2018.4-beta2',
          latestStable: '2018.3',
          nextUpgrade: '2018.3',
        }}
        accountExpiry={defaultExpiry}
        openExternalLink={() => {}}
        blockWhenDisconnected={false}
      />,
    );

    expect(component.state('type')).to.be.equal('update-available');
    expect(component.state('upgradeVersion')).to.be.equal('2018.3');
    expect(component.state('visible')).to.be.true;
  });

  it('handles beta update available', () => {
    const component = shallow(
      <NotificationArea
        tunnelState={{
          state: 'disconnected',
        }}
        version={{
          ...defaultVersion,
          upToDate: false,
          current: '2018.4-beta1',
          latest: '2018.4-beta3',
          latestStable: '2018.3',
          nextUpgrade: '2018.4-beta3',
        }}
        accountExpiry={defaultExpiry}
        openExternalLink={() => {}}
        blockWhenDisconnected={false}
      />,
    );

    expect(component.state('type')).to.be.equal('update-available');
    expect(component.state('upgradeVersion')).to.be.equal('2018.4-beta3');
    expect(component.state('visible')).to.be.true;
  });

  it('handles time running low', () => {
    const expiry = new AccountExpiry(
      moment()
        .add(2, 'days')
        .format(),
      'en',
    );
    const component = shallow(
      <NotificationArea
        tunnelState={{
          state: 'disconnected',
        }}
        version={defaultVersion}
        accountExpiry={expiry}
        openExternalLink={() => {}}
        blockWhenDisconnected={false}
      />,
    );

    expect(component.state('type')).to.be.equal('expires-soon');
    expect(component.state('visible')).to.be.true;
  });
});
