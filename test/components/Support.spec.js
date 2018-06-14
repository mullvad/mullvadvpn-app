// @flow

import { expect } from 'chai';
import React from 'react';
import Support from '../../app/components/Support';
import { shallow } from 'enzyme';
import sinon from 'sinon';
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
      onSend: (_report) => {},
    };
    return Object.assign({}, defaultProps, mergeProps);
  };

  it('should call close callback', (done) => {
    const props = makeProps({
      onClose: () => done(),
    });
    const component = getComponent(render(props), 'support__close');
    click(component);
  });

  it('should call view logs callback', (done) => {
    const props = makeProps({
      onViewLog: (_path) => done(),
    });
    const component = getComponent(render(props), 'support__view_logs');
    click(component);
  });

  it('should call send callback when description filled in', (done) => {
    const props = makeProps({
      onSend: (_report) => done(),
    });

    const component = render(props);
    component.setState({ message: 'abc', email: 'foo' });

    const sendButton = getComponent(component, 'support__send_logs');
    expect(sendButton.prop('disabled')).to.be.false;
    click(sendButton);
  });

  it('should not call send callback when description is empty', () => {
    const component = render(makeProps());
    component.setState({ message: '' });

    const sendButton = getComponent(render(makeProps()), 'support__send_logs');
    expect(sendButton.prop('disabled')).to.be.true;
  });

  it('should not collect report twice', (done) => {
    const collectCallback = sinon.spy(() => Promise.resolve('non-falsy'));
    const props = makeProps({
      onCollectLog: collectCallback,
    });

    const viewLogButton = getComponent(render(props), 'support__view_logs');
    click(viewLogButton);

    setTimeout(() => {
      click(viewLogButton);
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
      },
    });

    const component = render(props);
    component.setState({ message: '', email: 'foo' });

    const sendButton = getComponent(component, 'support__send_logs');
    click(sendButton);
  });
});

function render(props) {
  return shallow(<Support {...props} />);
}

function getComponent(container, testName) {
  return container.findWhere((n) => n.prop('testName') === testName);
}

function click(component) {
  component.prop('onPress')();
}
