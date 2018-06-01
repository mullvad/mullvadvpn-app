// @flow

import { expect } from 'chai';
import * as React from 'react';
import { shallow } from 'enzyme';
import sinon from 'sinon';

import Login from '../../app/components/Login';
import AccountInput from '../../app/components/AccountInput';

describe('components/Login', () => {
  it('notifies on the first change after failure', () => {
    let onFirstChange = sinon.spy();
    const props = {
      account: Object.assign({}, defaultAccount, {
        status: 'failed',
      }),
      onFirstChangeAfterFailure: onFirstChange,
    };

    const component = renderWithProps(props);
    const accountInput = component.find(AccountInput);

    accountInput.simulate('change', 'foo');
    expect(onFirstChange.calledOnce).to.be.true;

    onFirstChange.reset();

    accountInput.simulate('change', 'bar');
    expect(onFirstChange.calledOnce).to.be.false;
  });

  it('does not show the footer when logging in', () => {
    const component = renderLoggingIn();
    const visibleFooters = getComponent(component, 'footerVisibility true');
    const invisibleFooters = getComponent(component, 'footerVisibility false');
    expect(visibleFooters.length).to.equal(0);
    expect(invisibleFooters.length).to.equal(1);
  });

  it('shows the footer and account input when not logged in', () => {
    const component = renderNotLoggedIn();
    const visibleFooters = getComponent(component, 'footerVisibility true');
    const invisibleFooters = getComponent(component, 'footerVisibility false');
    expect(visibleFooters.length).to.equal(1);
    expect(invisibleFooters.length).to.equal(0);
    expect(getComponent(component, 'AccountInput').length).to.be.above(0);
  });

  it('does not show the footer nor account input when logged in', () => {
    const component = renderLoggedIn();
    const visibleFooters = getComponent(component, 'footerVisibility true');
    const invisibleFooters = getComponent(component, 'footerVisibility false');
    expect(visibleFooters.length).to.equal(0);
    expect(invisibleFooters.length).to.equal(1);
    expect(getComponent(component, 'AccountInput').length).to.equal(0);
  });

  it('logs in with the entered account number when clicking the login icon', (done) => {
    const component = renderNotLoggedIn();
    component.setProps({
      account: Object.assign({}, defaultAccount, {
        accountToken: '12345',
      }),
      onLogin: (an) => {
        try {
          expect(an).to.equal('12345');
          done();
        } catch (e) {
          done(e);
        }
      },
    });

    click(getComponent(component, 'account-input-button'));
  });
});

const defaultAccount = {
  accountToken: null,
  accountHistory: [],
  expiry: null,
  status: 'none',
  error: null,
};

const defaultProps = {
  account: defaultAccount,
  onLogin: () => {},
  onSettings: () => {},
  onChange: () => {},
  onFirstChangeAfterFailure: () => {},
  onExternalLink: () => {},
  onAccountTokenChange: (_accountToken) => {},
  onRemoveAccountTokenFromHistory: (_accountToken) => {},
};

function renderLoggedIn() {
  return renderWithProps({
    account: Object.assign(defaultAccount, {
      status: 'ok',
    }),
  });
}

function renderLoggingIn() {
  return renderWithProps({
    account: Object.assign(defaultAccount, {
      status: 'logging in',
    }),
  });
}

function renderNotLoggedIn() {
  return renderWithProps({
    account: Object.assign(defaultAccount, {
      status: 'none',
    }),
  });
}

function renderWithProps(customProps) {
  const props = Object.assign({}, defaultProps, customProps);
  return shallow(<Login {...props} />);
}

function getComponent(container, testName) {
  return container.findWhere((n) => n.prop('testName') === testName);
}

function click(component) {
  component.prop('onPress')();
}
