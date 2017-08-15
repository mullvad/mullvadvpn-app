// @flow

import { expect } from 'chai';
import React from 'react';
import { shallow } from 'enzyme';
import Login from '../../app/components/Login';
import AccountInput from '../../app/components/AccountInput';

import type { ShallowWrapper } from 'enzyme';

describe('components/Login', () => {

  it('notifies on the first change after failure', () => {

    let cbCalled = false;
    const props = {
      onFirstChangeAfterFailure: () => { cbCalled=true; },
    };

    const component = renderWithProps( props );
    const accountInput = component.find(AccountInput);

    // Change the props to a failed state
    component.setProps({ account: {
      status: 'failed',
    }});


    // Write something in the input field
    setInputText(accountInput, 'foo');
    expect(cbCalled).to.be.true;


    // Reset the test state
    cbCalled = false;

    // Write some other thing in the input field
    setInputText(accountInput, 'bar');
    expect(cbCalled).to.be.false;
  });

});

function renderWithProps(customProps): ShallowWrapper {
  const defaultProps = {
    account: {accountNumber: null,
      paidUntil: null,
      status: 'none',
      error: null,
    },
    onLogin: () => {},
    onSettings: () => {},
    onChange: () => {},
    onFirstChangeAfterFailure: () => {},
    onExternalLink: () => {},
  };
  const props = Object.assign({}, defaultProps, customProps);

  return shallow( <Login { ...props } /> );
}

function setInputText(input: ShallowWrapper, text: string) {
  input.simulate('change', {target: {value: text}});
}
