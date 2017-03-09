import { expect } from 'chai';
import { KeyType, createKeyEvent } from '../mocks/dom';

import React from 'react';
import ReactTestUtils from 'react-addons-test-utils';
import AccountInput from '../../app/components/AccountInput';

describe('components: AccountInput', () => {

  it('should call onEnter', (done) => {
    const onEnter = () => {
      done();
    };

    const component = ReactTestUtils.renderIntoDocument(
      <AccountInput onEnter={ onEnter } />
    );

    ReactTestUtils.Simulate.keyUp(component._ref, createKeyEvent(KeyType.Enter));
  });

  it('should call onChange', (done) => {
    const onChange = (val) => {
      expect(val).equal('1');
      done();
    };

    const component = ReactTestUtils.renderIntoDocument(
      <AccountInput onChange={ onChange } />
    );
    
    ReactTestUtils.Simulate.keyDown(component._ref, createKeyEvent(KeyType._1));
  });

  it('should remove last character', (done) => {
    const onChange = (val) => {
      expect(val).equal('123');
      done();
    };

    const component = ReactTestUtils.renderIntoDocument(
      <AccountInput value="1234" onChange={ onChange } />
    );

    ReactTestUtils.Simulate.keyDown(component._ref, createKeyEvent(KeyType.Backspace));
  });

});