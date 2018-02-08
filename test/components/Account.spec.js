// @flow

import { expect } from 'chai';
import React from 'react';
import { shallow } from 'enzyme';
require('../setup/enzyme');
import Account from '../../app/components/Account';

import type { AccountReduxState } from '../../app/redux/account/reducers';
import type { AccountProps } from '../../app/components/Account';

describe('components/Account', () => {
  const state: AccountReduxState = {
    accountToken: '1234',
    accountHistory: [],
    expiry: (new Date('2038-01-01')).toISOString(),
    status: 'none',
    error: null
  };

  const makeProps = (state: AccountReduxState, mergeProps: $Shape<AccountProps>): AccountProps => {
    const defaultProps: AccountProps = {
      account: state,
      onClose: () => {},
      onLogout: () => {},
      onBuyMore: () => {}
    };
    return Object.assign({}, defaultProps, mergeProps);
  };

  it('should call close callback', (done) => {
    const props = makeProps(state, {
      onClose: () => done()
    });
    const component = getComponent(render(props), 'account__close');
    click(component);
  });

  it('should call logout callback', (done) => {
    const props = makeProps(state, {
      onLogout: () => done()
    });
    const component = getComponent(render(props), 'account__logout');
    click(component);
  });

  it('should call "buy more" callback', (done) => {
    const props = makeProps(state, {
      onBuyMore: () => done()
    });
    const component = getComponent(render(props), 'account__buymore');
    click(component);
  });

  it('should display "out of time" message when account expired', () => {
    const expiredState: AccountReduxState = {
      accountToken: '1234',
      accountHistory: [],
      expiry: (new Date('2001-01-01')).toISOString(),
      status: 'none',
      error: null
    };
    const props = makeProps(expiredState, {});
    const component = getComponent(render(props), 'account__out_of_time');
    expect(component).to.have.length(1);
  });

  it('should not display "out of time" message when account is active', () => {
    const props = makeProps(state, {});
    const component = getComponent(render(props), 'account__out_of_time');
    expect(component).to.have.length(0);
  });

});

function render(props) {
  return shallow(
    <Account {...props} />
  );
}

function getComponent(container, testName) {
  return container.findWhere( n => n.prop('testName') === testName);
}

function click(component) {
  component.prop('onPress')();
}
