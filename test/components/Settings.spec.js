// @flow

import { expect } from 'chai';
import React from 'react';
import ReactTestUtils, { Simulate } from 'react-dom/test-utils';
import Settings from '../../app/components/Settings';
import { defaultServer } from '../../app/config';

import type { AccountReduxState } from '../../app/redux/account/reducers';
import type { SettingsReduxState } from '../../app/redux/settings/reducers';
import type { SettingsProps } from '../../app/components/Settings';

describe('components/Settings', () => {
  const loggedOutAccountState: AccountReduxState = {
    accountNumber: null,
    paidUntil: null,
    status: 'none',
    error: null
  };

  const loggedInAccountState: AccountReduxState = {
    accountNumber: '1234',
    paidUntil: (new Date('2038-01-01')).toISOString(),
    status: 'ok',
    error: null
  };

  const unpaidAccountState: AccountReduxState = {
    accountNumber: '1234',
    paidUntil: (new Date('2001-01-01')).toISOString(),
    status: 'ok',
    error: null
  };

  const settingsState: SettingsReduxState = {
    autoSecure: true,
    preferredServer: defaultServer
  };

  const makeProps = (anAccountState: AccountReduxState, aSettingsState: SettingsReduxState, mergeProps: $Shape<SettingsProps> = {}): SettingsProps => {
    const defaultProps: SettingsProps = {
      account: anAccountState,
      settings: aSettingsState,
      onQuit: () => {},
      onClose: () => {},
      onViewAccount: () => {},
      onExternalLink: (_type) => {},
      onUpdateSettings: (_update) => {}
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

  it('should call update callback', (done) => {
    const props = makeProps(loggedInAccountState, settingsState, {
      onUpdateSettings: (update) => {
        try {
          expect(update).to.include({ autoSecure: false });
          done();
        } catch(e) {
          done(e);
        }
      }
    });
    const domNode = ReactTestUtils.findRenderedDOMComponentWithClass(render(props), 'settings__autosecure');

    // TODO(Andrej): Add click handler to Switch to avoid calling that horrible chain of events.
    Simulate.mouseDown(domNode);
    Simulate.mouseUp(domNode);
    Simulate.change(domNode, { target: { checked: false } });
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

    expect(collectedExternalLinkTypes).to.include.ordered.members(['faq', 'guides', 'supportEmail']);
  });

});
