// @flow

import * as React from 'react';
import { shallow } from 'enzyme';
import { AccountInput } from '../src';

describe('components/AccountInput', () => {
  const getInputRef = (component) => {
    const node = getComponent(component, 'AccountInput');
    return node;
  };

  const render = (mergeProps: $Shape<React.ElementProps<typeof AccountInput>>) => {
    const defaultProps: React.ElementProps<typeof AccountInput> = {
      value: '',
      onEnter: null,
      onChange: null,
    };
    const props = Object.assign({}, defaultProps, mergeProps);
    return shallow(<AccountInput {...props} />);
  };

  it('should call onEnter', (done) => {
    const component = render({
      onEnter: () => done(),
    });
    keyPress(getInputRef(component), createKeyEvent('Enter'));
  });

  it('should call onChange', (done) => {
    const component = render({
      onChange: (val) => {
        expect(val).to.be.equal('1');
        done();
      },
    });
    keyPress(getInputRef(component), createKeyEvent('1'));
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
      '',
    ];

    for (const value of cases) {
      const component = render({ value });
      expect(getInputRef(component).prop('value')).to.be.equal(value);
    }
  });

  it('should remove last character', (done) => {
    const component = render({
      value: '1234',
      onChange: (val) => {
        expect(val).to.be.equal('123');
        done();
      },
    });
    keyPress(getInputRef(component), createKeyEvent('Backspace'));
  });

  it('should remove first character', (done) => {
    const component = render({
      value: '1234',
      onChange: (val) => {
        expect(val).to.be.equal('234');
        done();
      },
    });
    component.setState({ selectionRange: [1, 1] }, () => {
      keyPress(getInputRef(component), createKeyEvent('Backspace'));
    });
  });

  it('should remove all characters', (done) => {
    const component = render({
      value: '12345678',
      onChange: (val) => {
        expect(val).to.be.empty;
        done();
      },
    });
    component.setState({ selectionRange: [0, 8] }, () => {
      keyPress(getInputRef(component), createKeyEvent('Backspace'));
    });
  });

  it('should remove selection', (done) => {
    const component = render({
      value: '1234 5678 9999',
      onChange: (val) => {
        expect(val).to.be.equal('12349999');
        done();
      },
    });
    component.setState({ selectionRange: [4, 8] }, () => {
      keyPress(getInputRef(component), createKeyEvent('Backspace'));
    });
  });

  it('should replace selection', (done) => {
    const component = render({
      value: '0000',
    });

    component.setState({ selectionRange: [1, 3] }, () => {
      keyPress(getInputRef(component), createKeyEvent('1'));

      component.setState({}, () => {
        expect(component.state().value).to.be.equal('010');
        expect(component.state().selectionRange).to.deep.equal([2, 2]);
        done();
      });
    });
  });

  it('should keep selection in the back', (done) => {
    const component = render({ value: '' });

    for (let i = 0; i < 12; i++) {
      keyPress(getInputRef(component), createKeyEvent('1'));
    }

    component.setState({}, () => {
      expect(component.state().value).to.be.equal('111111111111');
      expect(component.state().selectionRange).to.deep.equal([12, 12]);
      done();
    });
  });

  it('should advance selection on insertion', (done) => {
    const component = render({
      value: '0000',
    });
    component.setState({ selectionRange: [1, 1] }, () => {
      keyPress(getInputRef(component), createKeyEvent('1'));

      component.setState({}, () => {
        expect(component.state().value).to.be.equal('01000');
        expect(component.state().selectionRange).to.deep.equal([2, 2]);
        done();
      });
    });
  });

  it('should not do anything when nothing to remove', (done) => {
    const component = render({
      value: '0000',
    });
    component.setState({ selectionRange: [0, 0] }, () => {
      keyPress(getInputRef(component), createKeyEvent('Backspace'));

      component.setState({}, () => {
        expect(component.state().value).to.be.equal('0000');
        expect(component.state().selectionRange).to.deep.equal([0, 0]);
        done();
      });
    });
  });
});

function getComponent(container, testName) {
  return container.findWhere((n) => n.prop('testName') === testName);
}

function keyPress(component, key) {
  component.prop('onKeyPress')(key);
}

// @flow
const keycodes = {
  '1': { which: 49, keyCode: 49 },
  '2': { which: 50, keyCode: 50 },
  '3': { which: 51, keyCode: 51 },
  '4': { which: 52, keyCode: 52 },
  '5': { which: 53, keyCode: 53 },
  '6': { which: 54, keyCode: 54 },
  '7': { which: 55, keyCode: 55 },
  '8': { which: 56, keyCode: 56 },
  '9': { which: 57, keyCode: 57 },
  '0': { which: 48, keyCode: 48 },
  Tab: { which: 9, keyCode: 9 },
  Enter: { which: 13, keyCode: 13 },
  Backspace: { which: 8, keyCode: 8 },
};

export type Keycode = $Keys<typeof keycodes>;

export function createKeyEvent(key: Keycode): Object {
  return Object.assign({}, { key }, keycodes[key], {
    // preventDefault is needed to mock key events for onKeyPress in AccountInput
    preventDefault: () => {},
  });
}
