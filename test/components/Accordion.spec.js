// @flow
/* eslint react/no-find-dom-node: off */

import { expect } from 'chai';
import * as React from 'react';
import ReactDOM from 'react-dom';
import Accordion from '../../app/components/Accordion';

describe('components/Accordion', () => {

  let container: ?HTMLElement;

  function renderIntoDocument(instance) {
    if(!container) {
      container = document.createElement('div');
      if(!document.documentElement) {
        throw new Error('document.documentElement cannot be null.');
      }
      document.documentElement.appendChild(container);
    }
    return ReactDOM.render(instance, container);
  }

  // unmount container and clean up DOM
  afterEach(() => {
    if(container) {
      ReactDOM.unmountComponentAtNode(container);
      container = null;
    }
  });

  it('should be collapsed upon mount', () => {
    const component = renderIntoDocument(
      <Accordion height={ 0 }>
        <div style={{ height: 100 }}></div>
      </Accordion>
    );
    const domNode = ReactDOM.findDOMNode(component);
    expect(domNode).to.have.property('clientHeight', 0);
  });

  it('should be expanded to provided height upon mount', () => {
    const component = renderIntoDocument(
      <Accordion height={ 100 } />
    );
    const domNode = ReactDOM.findDOMNode(component);
    expect(domNode).to.have.property('clientHeight', 100);
  });

  it('should be expanded using layout upon mount', () => {
    const component = renderIntoDocument(
      <Accordion height={ 'auto' }>
        <div style={{ height: 100 }}></div>
      </Accordion>
    );
    const domNode = ReactDOM.findDOMNode(component);
    expect(domNode).to.have.property('clientHeight', 100);
  });

  it('should collapse', () => {
    const component = renderIntoDocument(
      <Accordion height={ 'auto' }>
        <div style={{ height: 100 }}></div>
      </Accordion>
    );

    renderIntoDocument(
      <Accordion height={ 0 } transitionStyle="none">
        <div style={{ height: 100 }}></div>
      </Accordion>
    );

    const domNode = ReactDOM.findDOMNode(component);
    expect(domNode).to.have.property('clientHeight', 0);
  });

  it('should expand', () => {
    const component = renderIntoDocument(
      <Accordion height={ 0 }>
        <div style={{ height: 100 }}></div>
      </Accordion>
    );

    renderIntoDocument(
      <Accordion height="auto" transitionStyle="none">
        <div style={{ height: 100 }}></div>
      </Accordion>
    );

    const domNode = ReactDOM.findDOMNode(component);
    expect(domNode).to.have.property('clientHeight', 100);
  });

});