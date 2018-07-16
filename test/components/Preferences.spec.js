// @flow

import * as React from 'react';
import { shallow } from 'enzyme';
import Preferences from '../../app/components/Preferences';

describe('components/Preferences', () => {
  it('Should call close handler', (done) => {
    const props = makeProps({ onClose: done });
    const component = shallow(<Preferences {...props} />);
    const button = component.find({ testName: 'closeButton' });
    expect(button).to.have.length(1);
    button.simulate('press');
  });
});

function makeProps(props) {
  return {
    onClose: () => {},
    setAutoConnect: () => {},
    setAutoStart: (_autoStart) => Promise.resolve(),
    getAutoStart: () => false,
    setAllowLan: () => {},
    allowAutoConnect: false,
    allowLan: false,
    ...props,
  };
}
