// @flow

import { expect } from 'chai';
import sinon from 'sinon';
import React from 'react';
import ReactTestUtils, { Simulate } from 'react-dom/test-utils';
import Support from '../../app/components/Support';

import type { SupportProps } from '../../app/components/Support';

describe('components/Support', () => {

  const makeProps = (mergeProps: $Shape<SupportProps> = {}): SupportProps => {
    const defaultProps: SupportProps = {
      account: {
        accountToken: null,
        accountHistory: [],
        error: null,
        expiry: null,
        status: 'none',
      },
      onClose: () => {},
      onViewLog: (_path) => {},
      onCollectLog: () => Promise.resolve('/tmp/mullvad_problem_report.log'),
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
      onViewLog: (_path) => done()
    });
    const domNode = ReactTestUtils.findRenderedDOMComponentWithClass(render(props), 'support__form-view-logs');
    Simulate.click(domNode);
  });

  it('should call send callback when description filled in', (done) => {
    const props = makeProps({
      onSend: (_report) => done()
    });

    const component = render(props);

    const descriptionField = ReactTestUtils.findRenderedDOMComponentWithClass(component, 'support__form-message');
    descriptionField.value = 'Lorem Ipsum';
    Simulate.change(descriptionField);

    const sendButton = ReactTestUtils.findRenderedDOMComponentWithClass(component, 'support__form-send');
    expect(sendButton.disabled).to.be.false;
    Simulate.click(sendButton);
  });

  it('should not call send callback when description is empty', () => {
    const component = render(makeProps());

    const descriptionField = ReactTestUtils.findRenderedDOMComponentWithClass(component, 'support__form-message');
    descriptionField.value = '';
    Simulate.change(descriptionField);

    const sendButton = ReactTestUtils.findRenderedDOMComponentWithClass(component, 'support__form-send');
    expect(sendButton.disabled).to.be.true;
  });

  it('should not collect report twice', (done) => {
    const collectCallback = sinon.spy(() => Promise.resolve('non-falsy'));
    const props = makeProps({
      onCollectLog: collectCallback
    });

    const component = render(props);
    const viewLogButton = ReactTestUtils.findRenderedDOMComponentWithClass(component, 'support__form-view-logs');
    Simulate.click(viewLogButton);

    setTimeout(() => {
      Simulate.click(viewLogButton);
    });

    setTimeout(() => {
      try {
        expect(collectCallback.callCount).to.equal(1);
        done();
      } catch (e) {
        done(e);
      }
    });
  });

  it('should collect report on submission', (done) => {
    const collectCallback = sinon.spy(() => Promise.resolve(''));
    const props = makeProps({
      onCollectLog: collectCallback,
      onSend: (_report) => {
        try {
          expect(collectCallback.calledOnce).to.be.true;
          done();
        } catch (e) {
          done(e);
        }
      }
    });

    const component = render(props);

    const descriptionField = ReactTestUtils.findRenderedDOMComponentWithClass(component, 'support__form-message');
    descriptionField.value = 'Lorem Ipsum';
    Simulate.change(descriptionField);

    const sendButton = ReactTestUtils.findRenderedDOMComponentWithClass(component, 'support__form-send');
    Simulate.click(sendButton);
  });

});
