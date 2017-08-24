// @flow

import { expect } from 'chai';
import React from 'react';
import ReactTestUtils, { Simulate } from 'react-dom/test-utils';
import Account from '../../app/components/Account';

import type { AccountReduxState } from '../../app/redux/account/reducers';
import type { AccountProps } from '../../app/components/Account';

describe('components/Account', () => {
  const state: AccountReduxState = {
    accountNumber: '1234',
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

  const render = (props: AccountProps): Account => {
    return ReactTestUtils.renderIntoDocument(
      <Account { ...props } />
    );
  };

  it('should call close callback', (done) => {
    const props = makeProps(state, {
      onClose: () => done()
    });
    const domNode = ReactTestUtils.findRenderedDOMComponentWithClass(render(props), 'account__close');
    Simulate.click(domNode);
  });

  it('should call logout callback', (done) => {
    const props = makeProps(state, {
      onLogout: () => done()
    });
    const domNode = ReactTestUtils.findRenderedDOMComponentWithClass(render(props), 'account__logout');
    Simulate.click(domNode);
  });

  it('should call "buy more" callback', (done) => {
    const props = makeProps(state, {
      onBuyMore: () => done()
    });
    const domNode = ReactTestUtils.findRenderedDOMComponentWithClass(render(props), 'account__buymore');
    Simulate.click(domNode);
  });

  it('should display "out of time" message when account expired', () => {
    const expiredState: AccountReduxState = {
      accountNumber: '1234',
      expiry: (new Date('2001-01-01')).toISOString(),
      status: 'none',
      error: null
    };
    const props = makeProps(expiredState, {});
    ReactTestUtils.findRenderedDOMComponentWithClass(render(props), 'account__out-of-time');
  });

  it('should not display "out of time" message when account is active', () => {
    const props = makeProps(state, {});
    const domNodes = ReactTestUtils.scryRenderedDOMComponentsWithClass(render(props), 'account__out-of-time');
    expect(domNodes.length).to.be.equal(0);
  });

});