import moment from 'moment';
import * as React from 'react';
import { shallow } from 'enzyme';
import NotificationArea from '../../src/renderer/components/NotificationArea';
import AccountExpiry from '../../src/renderer/lib/account-expiry';

describe('components/NotificationArea', () => {
  const defaultVersion = {
    consistent: true,
    currentIsSupported: true,
    upToDate: true,
    current: '2018.2',
    latest: '2018.2-beta1',
    latestStable: '2018.2',
    nextUpgrade: null,
  };

  const defaultExpiry = new AccountExpiry(
    moment()
      .add(1, 'year')
      .format(),
  );

  it('handles disconnecting state', () => {
    for (const reason of ['nothing', 'block', 'reconnect']) {
      const component = shallow(
        <NotificationArea
          tunnelState={{
            state: 'disconnecting',
            details: { reason },
          }}
          version={defaultVersion}
          accountExpiry={defaultExpiry}
        />,
      );
      expect(component.state('visible')).to.be.false;
    }
  });

  it('handles connected or disconnected states', () => {
    for (const state of ['connected', 'disconnected']) {
      const component = shallow(
        <NotificationArea
          tunnelState={{
            state,
          }}
          version={defaultVersion}
          accountExpiry={defaultExpiry}
        />,
      );

      expect(component.state('visible')).to.be.false;
    }
  });

  it('handles connecting state', () => {
    const component = shallow(
      <NotificationArea
        tunnelState={{
          state: 'connecting',
        }}
        version={defaultVersion}
        accountExpiry={defaultExpiry}
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
    );
    const component = shallow(
      <NotificationArea
        tunnelState={{
          state: 'disconnected',
        }}
        version={defaultVersion}
        accountExpiry={expiry}
      />,
    );

    expect(component.state('type')).to.be.equal('expires-soon');
    expect(component.state('visible')).to.be.true;
  });
});
