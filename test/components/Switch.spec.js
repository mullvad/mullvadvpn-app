// @flow

import { expect } from 'chai';
import React from 'react';
import ReactTestUtils, { Simulate } from 'react-dom/test-utils';
import Switch from '../../app/components/Switch';

describe('components/Switch', () => {

  it('should switch on', (done) => {
    const onChange = (isOn) => {
      expect(isOn).to.be.true;
      done();
    };
    const component = ReactTestUtils.renderIntoDocument(
      <Switch isOn={ false } onChange={ onChange } />
    );
    const domNode = ReactTestUtils.findRenderedDOMComponentWithTag(component, 'input');

    Simulate.mouseDown(domNode, { clientX: 100, clientY: 0 });
    Simulate.mouseUp(domNode, { clientX: 100, clientY: 0 });

    domNode.checked = true;
    Simulate.change(domNode);
  });

  it('should switch off', (done) => {
    const onChange = (isOn) => {
      expect(isOn).to.be.false;
      done();
    };
    const component = ReactTestUtils.renderIntoDocument(
      <Switch isOn={ true } onChange={ onChange } />
    );
    const domNode = ReactTestUtils.findRenderedDOMComponentWithTag(component, 'input');

    Simulate.mouseDown(domNode, { clientX: 100, clientY: 0 });
    Simulate.mouseUp(domNode, { clientX: 100, clientY: 0 });

    domNode.checked = false;
    Simulate.change(domNode);
  });

  it('should handle left to right swipe', (done) => {
    const onChange = (isOn) => {
      expect(isOn).to.be.true;
      done();
    };
    const component = ReactTestUtils.renderIntoDocument(
      <Switch isOn={ false } onChange={ onChange } />
    );
    const domNode = ReactTestUtils.findRenderedDOMComponentWithTag(component, 'input');

    Simulate.mouseDown(domNode, { clientX: 100, clientY: 0 });

    // Switch listens to events on document
    document.dispatchEvent(new MouseEvent('mousemove', { clientX: 150, clientY: 0 }));
    document.dispatchEvent(new MouseEvent('mouseup', { clientX: 150, clientY: 0 }));
  });

  it('should handle right to left swipe', (done) => {
    const onChange = (isOn) => {
      expect(isOn).to.be.false;
      done();
    };
    const component = ReactTestUtils.renderIntoDocument(
      <Switch isOn={ true } onChange={ onChange } />
    );
    const domNode = ReactTestUtils.findRenderedDOMComponentWithTag(component, 'input');

    Simulate.mouseDown(domNode, { clientX: 150, clientY: 0 });

    // Switch listens to events on document
    document.dispatchEvent(new MouseEvent('mousemove', { clientX: 100, clientY: 0 }));
    document.dispatchEvent(new MouseEvent('mouseup', { clientX: 100, clientY: 0 }));
  });

  it('should timeout when user holds knob for too long without moving', (done) => {
    const onChange = () => {
      throw new Error('onChange should not be called on timeout.');
    };

    const component = ReactTestUtils.renderIntoDocument(
      <Switch isOn={ false } onChange={ onChange } />
    );
    const domNode = ReactTestUtils.findRenderedDOMComponentWithTag(component, 'input');

    Simulate.mouseDown(domNode, { clientX: 100, clientY: 0 });

    setTimeout(() => {
      // Switch listens to events on document
      document.dispatchEvent(new MouseEvent('mouseup', { clientX: 100, clientY: 0 }));

      try {
        // should not trigger onChange()
        Simulate.change(domNode);
        done();
      } catch(e) {
        done(e);
      }
    }, 1000);
  });

});