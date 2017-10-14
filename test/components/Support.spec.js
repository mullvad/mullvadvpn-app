// @flow

import { expect } from 'chai';
import React from 'react';
import ReactTestUtils, { Simulate } from 'react-dom/test-utils';
import Support from '../../app/components/Support';

import type { SupportProps } from '../../app/components/Support';

describe('components/Support', () => {

  const makeProps = (mergeProps: $Shape<SupportProps> = {}): SupportProps => {
    const defaultProps: SupportProps = {
      onClose: () => {},
      onViewLogs: () => {},
      onSend: (_report) => {}
    };
    return Object.assign({}, defaultProps, mergeProps);
  };

  const render = (props: SupportProps): Support => {
    return ReactTestUtils.renderIntoDocument(
      <Support { ...props } />
    );
  };

  it('should call close callback', (done) => {
    const props = makeProps({
      onClose: () => done()
    });
    const domNode = ReactTestUtils.findRenderedDOMComponentWithClass(render(props), 'support__close');
    Simulate.click(domNode);
  });

  it('should call view logs callback', (done) => {
    const props = makeProps({
      onViewLogs: () => done()
    });
    const domNode = ReactTestUtils.findRenderedDOMComponentWithClass(render(props), 'support__form-view-logs');
    Simulate.click(domNode);
  });

  it('should call send callback when description filled in', (done) => {
    const props = makeProps({
      onSend: (_report) => done()
    });

    const component = render(props);

    const descriptionField = ReactTestUtils.findRenderedDOMComponentWithClass(component, 'support__form-description');
    descriptionField.value = 'Lorem Ipsum';
    Simulate.change(descriptionField);

    const sendButton = ReactTestUtils.findRenderedDOMComponentWithClass(component, 'support__form-send');
    expect(sendButton.disabled).to.be.false;
    Simulate.click(sendButton);
  });

  it('should not call send callback when description is empty', () => {
    const component = render(makeProps());

    const descriptionField = ReactTestUtils.findRenderedDOMComponentWithClass(component, 'support__form-description');
    descriptionField.value = '';
    Simulate.change(descriptionField);

    const sendButton = ReactTestUtils.findRenderedDOMComponentWithClass(component, 'support__form-send');
    expect(sendButton.disabled).to.be.true;
  });

});
