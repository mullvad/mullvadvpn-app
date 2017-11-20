// @flow

import { expect } from 'chai';
import React from 'react';
import ReactTestUtils, { Simulate } from 'react-dom/test-utils';
import Settings from '../../app/components/Settings';

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
      host: 'any',
      port: 'any',
      protocol: 'any',
    },
  };

  const makeProps = (anAccountState: AccountReduxState, aSettingsState: SettingsReduxState, mergeProps: $Shape<SettingsProps> = {}): SettingsProps => {
    const defaultProps: SettingsProps = {
      account: anAccountState,
      settings: aSettingsState,
      onQuit: () => {},
      onClose: () => {},
      onViewAccount: () => {},
      onViewSupport: () => {},
      onViewAdvancedSettings: () => {},
      onExternalLink: (_type) => {}
    };
    return Object.assign({}, defaultProps, mergeProps);
  };

  const render = (props: SettingsProps): Settings => {
    return ReactTestUtils.renderIntoDocument(
      <Settings { ...props } />
    );
  };

  it('should show quit button when logged out', () => {
    const props = makeProps(loggedOutAccountState, settingsState);
    ReactTestUtils.findRenderedDOMComponentWithClass(render(props), 'settings__quit');
  });

  it('should show quit button when logged in', () => {
    const props = makeProps(loggedInAccountState, settingsState);
    ReactTestUtils.findRenderedDOMComponentWithClass(render(props), 'settings__quit');
  });

  it('should show external links when logged out', () => {
    const props = makeProps(loggedOutAccountState, settingsState);
    ReactTestUtils.findRenderedDOMComponentWithClass(render(props), 'settings__external');
  });

  it('should show external links when logged in', () => {
    const props = makeProps(loggedInAccountState, settingsState);
    ReactTestUtils.findRenderedDOMComponentWithClass(render(props), 'settings__external');
  });

  it('should show account section when logged in', () => {
    const props = makeProps(loggedInAccountState, settingsState);
    ReactTestUtils.findRenderedDOMComponentWithClass(render(props), 'settings__account');
  });

  it('should hide account section when logged out', () => {
    const props = makeProps(loggedOutAccountState, settingsState);
    const elements = ReactTestUtils.scryRenderedDOMComponentsWithClass(render(props), 'settings__account');
    expect(elements).to.be.empty;
  });

  it('should hide account link when not logged in', () => {
    const props = makeProps(loggedOutAccountState, settingsState);
    const elements = ReactTestUtils.scryRenderedDOMComponentsWithClass(render(props), 'settings__view-account');
    expect(elements).to.be.empty;
  });

  it('should show out-of-time message for unpaid account', () => {
    const props = makeProps(unpaidAccountState, settingsState);
    const domNode = ReactTestUtils.findRenderedDOMComponentWithClass(render(props), 'settings__account-paid-until-label');
    expect(domNode.textContent).to.contain('OUT OF TIME');
  });

  it('should hide out-of-time message for paid account', () => {
    const props = makeProps(loggedInAccountState, settingsState);
    const domNode = ReactTestUtils.findRenderedDOMComponentWithClass(render(props), 'settings__account-paid-until-label');
    expect(domNode.textContent).to.not.contain('OUT OF TIME');
  });

  it('should call close callback', (done) => {
    const props = makeProps(loggedOutAccountState, settingsState, {
      onClose: () => done()
    });
    const domNode = ReactTestUtils.findRenderedDOMComponentWithClass(render(props), 'settings__close');
    Simulate.click(domNode);
  });

  it('should call quit callback', (done) => {
    const props = makeProps(loggedOutAccountState, settingsState, {
      onQuit: () => done()
    });
    const domNode = ReactTestUtils.findRenderedDOMComponentWithClass(render(props), 'settings__quit');
    Simulate.click(domNode);
  });

  it('should call account callback', (done) => {
    const props = makeProps(loggedInAccountState, settingsState, {
      onViewAccount: () => done()
    });
    const domNode = ReactTestUtils.findRenderedDOMComponentWithClass(render(props), 'settings__view-account');
    Simulate.click(domNode);
  });

  it('should call support callback', (done) => {
    const props = makeProps(loggedInAccountState, settingsState, {
      onViewSupport: () => done()
    });
    const domNode = ReactTestUtils.findRenderedDOMComponentWithClass(render(props), 'settings__view-support');
    Simulate.click(domNode);
  });

  it('should call external links callback', () => {
    let collectedExternalLinkTypes: Array<string> = [];
    const props = makeProps(loggedOutAccountState, settingsState, {
      onExternalLink: (type) => {
        collectedExternalLinkTypes.push(type);
      }
    });
    const container = ReactTestUtils.findRenderedDOMComponentWithClass(render(props), 'settings__external');
    Array.from(container.childNodes)
      .filter((elm: HTMLElement) => elm.classList.contains('settings__cell'))
      .forEach((elm) => Simulate.click(elm));

    expect(collectedExternalLinkTypes).to.include.ordered.members(['faq', 'guides']);
  });

});
