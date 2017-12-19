// @flow

import { expect } from 'chai';
import React from 'react';
import { shallow } from 'enzyme';
import sinon from 'sinon';

import Login from '../../app/components/Login';
import AccountInput from '../../app/components/AccountInput';

import type { ShallowWrapper } from 'enzyme';

describe('components/Login', () => {

  it('notifies on the first change after failure', () => {

    let callback = sinon.spy();
    const props = {
      onFirstChangeAfterFailure: callback,
    };

    const component = renderWithProps( props );
    const accountInput = component.find(AccountInput);

    // Change the props to a failed state
    component.setProps({ account: {
      status: 'failed',
    }});


    // Write something in the input field
    setInputText(accountInput, 'foo');
    expect(callback.calledOnce).to.be.true;


    // Reset the test state
    callback.reset();

    // Write some other thing in the input field
    setInputText(accountInput, 'bar');
    expect(callback.calledOnce).to.be.false;
  });

  it('doesn\'t show the footer when logging in', () => {
    const component = renderLoggingIn();

    const footer = component.find('.login-footer');
    expect(footer.hasClass('login-footer--invisible')).to.be.true;
  });

  it('shows the footer and account input when not logged in', () => {
    const component = renderNotLoggedIn();

    const footer = component.find('.login-footer');
    expect(footer.hasClass('login-footer--invisible')).to.be.false;
    expect(component.find(AccountInput).exists()).to.be.true;
  });

  it('doesn\'t show the footer nor account input when logged in', () => {
    const component = renderLoggedIn();

    const footer = component.find('.login-footer');
    expect(footer.hasClass('login-footer--invisible')).to.be.true;
    expect(component.find('.login-form__fields').hasClass('login-form__fields--invisible')).to.be.true;
  });

  it('logs in with the entered account number when clicking the login icon', (done) => {
    const component = renderNotLoggedIn();
    component.setProps({
      account: {
        accountToken: '12345',
      },
      onLogin: (an) => {
        try {
          expect(an).to.equal('12345');
          done();
        } catch (e) {
          done(e);
        }
      },
    });

    component.find('.login-form__submit').simulate('click');
  });
});

const defaultAccount = {
  accountToken: null,
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
  onAccountTokenChange: () => {},
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

function renderWithProps(customProps): ShallowWrapper {
  const props = Object.assign({}, defaultProps, customProps);
  return shallow( <Login { ...props } /> );
}

function setInputText(input: ShallowWrapper, text: string) {
  input.simulate('change', text);
}
