// @flow

import * as React from 'react';
import ReactDOM from 'react-dom';
import ReactTestUtils, { Simulate } from 'react-dom/test-utils';
import Switch from '../../src/renderer/components/Switch';

describe('components/Switch', () => {
  let container: ?HTMLElement;

  function renderIntoDocument(instance: React.Element<*>): React.Component<*, *> {
    if (container) {
      throw new Error('Unmount previously rendered component first.');
    }

    container = document.createElement('div');
    if (!document.documentElement) {
      throw new Error('document.documentElement cannot be null.');
    }

    document.documentElement.appendChild(container);

    return ReactDOM.render(instance, container);
  }

  // unmount container and clean up DOM
  afterEach(() => {
    if (container) {
      ReactDOM.unmountComponentAtNode(container);
      container = null;
    }
  });

  it('should switch on', (done) => {
    const onChange = (isOn) => {
      expect(isOn).to.be.true;
      done();
    };
    const component = renderIntoDocument(<Switch isOn={false} onChange={onChange} />);
    const domNode = ReactTestUtils.findRenderedDOMComponentWithTag(component, 'input');
    // See: https://github.com/facebook/flow/pull/5841
    if (domNode) {
      Simulate.mouseDown(domNode, { clientX: 100, clientY: 0 });
      Simulate.mouseUp(domNode, { clientX: 100, clientY: 0 });
      Simulate.change(domNode, { target: { checked: true } });
    }
  });

  it('should switch off', (done) => {
    const onChange = (isOn) => {
      expect(isOn).to.be.false;
      done();
    };
    const component = renderIntoDocument(<Switch isOn={true} onChange={onChange} />);
    const domNode = ReactTestUtils.findRenderedDOMComponentWithTag(component, 'input');
    // See: https://github.com/facebook/flow/pull/5841
    if (domNode) {
      Simulate.mouseDown(domNode, { clientX: 100, clientY: 0 });
      Simulate.mouseUp(domNode, { clientX: 100, clientY: 0 });
      Simulate.change(domNode, { target: { checked: false } });
    }
  });

  it('should handle left to right swipe', (done) => {
    const onChange = (isOn) => {
      expect(isOn).to.be.true;
      done();
    };
    const component = renderIntoDocument(<Switch isOn={false} onChange={onChange} />);
    const domNode = ReactTestUtils.findRenderedDOMComponentWithTag(component, 'input');
    // See: https://github.com/facebook/flow/pull/5841
    if (domNode) {
      Simulate.mouseDown(domNode, { clientX: 100, clientY: 0 });
    }

    // Switch listens to events on document
    document.dispatchEvent(new MouseEvent('mousemove', { clientX: 150, clientY: 0 }));
    document.dispatchEvent(new MouseEvent('mouseup', { clientX: 150, clientY: 0 }));
  });

  it('should handle right to left swipe', (done) => {
    const onChange = (isOn) => {
      expect(isOn).to.be.false;
      done();
    };
    const component = renderIntoDocument(<Switch isOn={true} onChange={onChange} />);
    const domNode = ReactTestUtils.findRenderedDOMComponentWithTag(component, 'input');

    // See: https://github.com/facebook/flow/pull/5841
    if (domNode) {
      Simulate.mouseDown(domNode, { clientX: 150, clientY: 0 });
    }

    // Switch listens to events on document
    document.dispatchEvent(new MouseEvent('mousemove', { clientX: 100, clientY: 0 }));
    document.dispatchEvent(new MouseEvent('mouseup', { clientX: 100, clientY: 0 }));
  });

  it('should timeout when user holds knob for too long without moving', (done) => {
    const onChange = () => {
      throw new Error('onChange should not be called on timeout.');
    };

    const component = renderIntoDocument(<Switch isOn={false} onChange={onChange} />);

    const domNode = ReactTestUtils.findRenderedDOMComponentWithTag(component, 'input');
    // See: https://github.com/facebook/flow/pull/5841
    if (domNode) {
      Simulate.mouseDown(domNode, { clientX: 100, clientY: 0 });
    }

    setTimeout(() => {
      // Switch listens to events on document
      document.dispatchEvent(new MouseEvent('mouseup', { clientX: 100, clientY: 0 }));

      try {
        // See: https://github.com/facebook/flow/pull/5841
        if (domNode) {
          // should not trigger onChange()
          Simulate.change(domNode);
        }
        done();
      } catch (e) {
        done(e);
      }
    }, 1000);
  });
});
