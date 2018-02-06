// @flow

import { expect } from 'chai';
import React from 'react';
import Settings from '../../app/components/Settings';

import { shallow } from 'enzyme';
require('../setup/enzyme');

import type { AccountReduxState } from '../../app/redux/account/reducers';
import type { SettingsReduxState } from '../../app/redux/settings/reducers';
import type { SettingsProps } from '../../app/components/Settings';

describe('components/Settings', () => {
  const loggedOutAccountState: AccountReduxState = {
    accountToken: null,
    accountHistory: [],
    expiry: null,
    status: 'none',
    error: null
  };

  const loggedInAccountState: AccountReduxState = {
    accountToken: '1234',
    accountHistory: [],
    expiry: (new Date('2038-01-01')).toISOString(),
    status: 'ok',
    error: null
  };

  const unpaidAccountState: AccountReduxState = {
    accountToken: '1234',
    accountHistory: [],
    expiry: (new Date('2001-01-01')).toISOString(),
    status: 'ok',
    error: null
  };

  const settingsState: SettingsReduxState = {
    relaySettings: {
      normal: {
        location: 'any',
        protocol: 'udp',
        port: 1301,
      },
    },
    relayLocations: [],
    allowLan: false,
  };

  const makeProps = (anAccountState: AccountReduxState, aSettingsState: SettingsReduxState, mergeProps: $Shape<SettingsProps> = {}): SettingsProps => {
    const defaultProps: SettingsProps = {
      account: anAccountState,
      settings: aSettingsState,
      version: '',
      onQuit: () => {},
      onClose: () => {},
      onViewAccount: () => {},
      onViewSupport: () => {},
      onViewAdvancedSettings: () => {},
      onViewPreferences: () => {},
      onExternalLink: (_type) => {}
    };
    return Object.assign({}, defaultProps, mergeProps);
  };

  it('should show quit button when logged out', () => {
    const props = makeProps(loggedOutAccountState, settingsState);
    const component = getComponent(render(props), 'settings__quit');
    expect(component).to.have.length(1);
  });

  it('should show quit button when logged in', () => {
    const props = makeProps(loggedInAccountState, settingsState);
    const component = getComponent(render(props), 'settings__quit');
    expect(component).to.have.length(1);
  });

  it('should show external links when logged out', () => {
    const props = makeProps(loggedOutAccountState, settingsState);
    const component = getComponent(render(props), 'settings__external_link');
    expect(component.length).to.be.above(0);
  });

  it('should show external links when logged in', () => {
    const props = makeProps(loggedInAccountState, settingsState);
    const component = getComponent(render(props), 'settings__external_link');
    expect(component.length).to.be.above(0);
  });

  it('should show account section when logged in', () => {
    const props = makeProps(loggedInAccountState, settingsState);
    const component = getComponent(render(props), 'settings__account');
    expect(component).to.have.length(1);
  });

  it('should hide account section when logged out', () => {
    const props = makeProps(loggedOutAccountState, settingsState);
    const elements = getComponent(render(props), 'settings__account');
    expect(elements).to.have.length(0);
  });

  it('should hide account link when not logged in', () => {
    const props = makeProps(loggedOutAccountState, settingsState);
    const elements = getComponent(render(props), 'settings__view_account');
    expect(elements).to.have.length(0);
  });

  it('should show out-of-time message for unpaid account', () => {
    const props = makeProps(unpaidAccountState, settingsState);
    const component = getComponent(render(props), 'settings__account_paid_until_label');
    expect(component.prop('subtext')).to.equal('OUT OF TIME');
  });

  it('should hide out-of-time message for paid account', () => {
    const props = makeProps(loggedInAccountState, settingsState);
    const component = getComponent(render(props), 'settings__account_paid_until_label');
    expect(component.prop('subtext')).not.to.equal('OUT OF TIME');
  });

  it('should call close callback', (done) => {
    const props = makeProps(loggedOutAccountState, settingsState, {
      onClose: () => done()
    });
    const component = getComponent(render(props), 'settings__close');
    click(component);
  });

  it('should call quit callback', (done) => {
    const props = makeProps(loggedOutAccountState, settingsState, {
      onQuit: () => done()
    });
    const component = getComponent(render(props), 'settings__quit');
    click(component);
  });

  it('should call account callback', (done) => {
    const props = makeProps(loggedInAccountState, settingsState, {
      onViewAccount: () => done()
    });
    const component = getComponent(render(props), 'settings__account_paid_until_label');
    click(component);
  });

  it('should call advanced settings callback', (done) => {
    const props = makeProps(loggedInAccountState, settingsState, {
      onViewAdvancedSettings: () => done()
    });
    const component = getComponent(render(props), 'settings__advanced');
    click(component);
  });

  it('should call preferences callback', (done) => {
    const props = makeProps(loggedInAccountState, settingsState, {
      onViewPreferences: () => done()
    });
    const component = getComponent(render(props), 'settings__preferences');
    click(component);
  });

  it('should call support callback', (done) => {
    const props = makeProps(loggedInAccountState, settingsState, {
      onViewSupport: () => done()
    });
    const component = getComponent(render(props), 'settings__view_support');
    click(component);
  });

  it('should call external links callback', () => {
    let collectedExternalLinkTypes: Array<string> = [];
    const props = makeProps(loggedOutAccountState, settingsState, {
      onExternalLink: (type) => {
        collectedExternalLinkTypes.push(type);
      }
    });
    const container = getComponent(render(props), 'settings__external_link');
    container.find({ testName: 'settings__external_link' })
      .forEach((element) => click(element));

    expect(collectedExternalLinkTypes).to.include.ordered.members(['faq', 'guides']);
  });

});

function render(props) {
  return shallow(
    <Settings {...props} />
  );
}

function getComponent(container, testName) {
  return container.findWhere( n => n.prop('testName') === testName);
}

function click(component) {
  component.prop('onPress')();
}
