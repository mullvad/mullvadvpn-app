import { expect } from 'chai';
import { KeyType, createKeyEvent } from '../mocks/dom';

import React from 'react';
import ReactTestUtils, { Simulate } from 'react-addons-test-utils';
import AccountInput from '../../app/components/AccountInput';

describe('components/AccountInput', () => {

  it('should call onEnter', (done) => {
    const onEnter = () => {
      done();
    };

    const component = ReactTestUtils.renderIntoDocument(
      <AccountInput onEnter={ onEnter } />
    );

    Simulate.keyUp(component._ref, createKeyEvent(KeyType.Enter));
  });

  it('should call onChange', (done) => {
    const onChange = (val) => {
      expect(val).to.be.equal('1');
      done();
    };

    const component = ReactTestUtils.renderIntoDocument(
      <AccountInput onChange={ onChange } />
    );
    
    Simulate.keyDown(component._ref, createKeyEvent(KeyType._1));
  });

  it('should format input properly', () => {
    const cases = [
      '1111111111111',
      '1111 1111 1111',
      '1111 1111 111',
      '1111 1111 11',
      '1111 1111 1',
      '1111 1111',
      '1111 111',
      '1111 11',
      '1111 1',
      '1111',
      '111',
      '11',
      '1',
      ''
    ];

    for(const val of cases) {
      const component = ReactTestUtils.renderIntoDocument(
        <AccountInput value={ val } />
      );
      expect(component._ref.value).to.be.equal(val);
    }
  });

  it('should remove last character', (done) => {
    const onChange = (val) => {
      expect(val).to.be.equal('123');
      done();
    };

    const component = ReactTestUtils.renderIntoDocument(
      <AccountInput value="1234" onChange={ onChange } />
    );

    Simulate.keyDown(component._ref, createKeyEvent(KeyType.Backspace));
  });

  it('should remove first character', (done) => {
    const onChange = (val) => {
      expect(val).to.be.equal('234');
      done();
    };

    const component = ReactTestUtils.renderIntoDocument(
      <AccountInput value="1234" onChange={ onChange } />
    );

    component.setState({ selectionRange: [1, 1] }, () => {
      Simulate.keyDown(component._ref, createKeyEvent(KeyType.Backspace));
    });
  });

  it('should remove all characters', (done) => {
    const onChange = (val) => {
      expect(val).to.be.empty;
      done();
    };

    const component = ReactTestUtils.renderIntoDocument(
      <AccountInput value="12345678" onChange={ onChange } />
    );

    component.setState({ selectionRange: [0, 8] }, () => {
      Simulate.keyDown(component._ref, createKeyEvent(KeyType.Backspace));
    });
  });

  it('should remove selection', (done) => {
    const onChange = (val) => {
      expect(val).to.be.equal('12349999');
      done();
    };

    const component = ReactTestUtils.renderIntoDocument(
      <AccountInput value="1234 5678 9999" onChange={ onChange } />
    );

    component.setState({ selectionRange: [4, 8] }, () => {
      Simulate.keyDown(component._ref, createKeyEvent(KeyType.Backspace));
    });
  });

  it('should replace selection', (done) => {
    const component = ReactTestUtils.renderIntoDocument(
      <AccountInput value="0000" />
    );
    
    component.setState({ selectionRange: [1, 3] }, () => {
      Simulate.keyDown(component._ref, createKeyEvent(KeyType._1));

      component.setState({}, () => {
        expect(component.state.value).to.be.equal('010');
        expect(component.state.selectionRange).to.deep.equal([2, 2]);
        done();
      });
    });
  });

  it('should keep selection in the back', (done) => {
    const component = ReactTestUtils.renderIntoDocument(
      <AccountInput />
    );

    for(let i = 0; i < 12; i++) {
      Simulate.keyDown(component._ref, createKeyEvent(KeyType._1));
    }
    
    component.setState({}, () => {
      expect(component.state.value).to.be.equal('111111111111');
      expect(component.state.selectionRange).to.deep.equal([12, 12]);
      done();
    });
  });

  it('should advance selection on insertion', (done) => {
    const component = ReactTestUtils.renderIntoDocument(
      <AccountInput value="0000" />
    );
    
    component.setState({ selectionRange: [1, 1]}, () => {
      Simulate.keyDown(component._ref, createKeyEvent(KeyType._1));
      
      component.setState({}, () => {
        expect(component.state.value).to.be.equal('01000');
        expect(component.state.selectionRange).to.deep.equal([2, 2]);
        done();
      });
    });
  });

  it('should not do anything when nothing to remove', (done) => {
    const component = ReactTestUtils.renderIntoDocument(
      <AccountInput value="0000" />
    );
    
    component.setState({ selectionRange: [0, 0] }, () => {
      Simulate.keyDown(component._ref, createKeyEvent(KeyType.Backspace));

      component.setState({}, () => {
        expect(component.state.value).to.be.equal('0000');
        expect(component.state.selectionRange).to.deep.equal([0, 0]);
        done();
      });
    });
  });

});