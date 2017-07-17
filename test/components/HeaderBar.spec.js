// @flow

import { expect } from 'chai';
import React from 'react';
import ReactTestUtils, { Simulate } from 'react-dom/test-utils';
import HeaderBar from '../../app/components/HeaderBar';

describe('components/HeaderBar', () => {

  it('should display headerbar', () => {
    const component = ReactTestUtils.renderIntoDocument(
      <HeaderBar hidden={ false } />
    );
    ReactTestUtils.findRenderedDOMComponentWithClass(component, 'headerbar__container');
  });

  it('should not display headerbar', () => {
    const component = ReactTestUtils.renderIntoDocument(
      <HeaderBar hidden={ true } />
    );
    const domNodes = ReactTestUtils.scryRenderedDOMComponentsWithClass(component, 'headerbar__container');
    expect(domNodes.length).to.be.equal(0);
  });

  it('should display settings button', () => {
    const component = ReactTestUtils.renderIntoDocument(
      <HeaderBar showSettings={ true } />
    );
    ReactTestUtils.findRenderedDOMComponentWithClass(component, 'headerbar__settings');
  });

  it('should not display settings button', () => {
    const component = ReactTestUtils.renderIntoDocument(
      <HeaderBar showSettings={ false } />
    );
    const domNodes = ReactTestUtils.scryRenderedDOMComponentsWithClass(component, 'headerbar__settings');
    expect(domNodes.length).to.be.equal(0);
  });

  it('should call settings callback', (done) => {
    const component = ReactTestUtils.renderIntoDocument(
      <HeaderBar showSettings={ true } onSettings={ () => done() } />
    );
    const domNode = ReactTestUtils.findRenderedDOMComponentWithClass(component, 'headerbar__settings');
    Simulate.click(domNode);
  });

});