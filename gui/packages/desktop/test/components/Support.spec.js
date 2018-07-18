// @flow

import * as React from 'react';
import { shallow } from 'enzyme';
import Support from '../../src/renderer/components/Support';
import type { SupportProps } from '../../src/renderer/components/Support';
import { BackBarItem } from '../../src/renderer/components/NavigationBar';

describe('components/Support', () => {
  it('should call close callback', () => {
    const props = makeProps({ onClose: spy() });
    const component = shallow(<Support {...props} />);

    const closeButton = component.find(BackBarItem).dive();
    closeButton.simulate('press');

    expect(props.onClose).to.have.been.called.once;
  });

  it('should call view logs callback', async () => {
    const props = makeProps({ viewLog: spy() });
    const component = shallow(<Support {...props} />);
    const viewButton = component.find({ testName: 'support__view_logs' });

    await click(viewButton);
    expect(props.viewLog).to.have.been.called.once;
  });

  it('should call send callback when description filled in', async () => {
    const props = makeProps({
      defaultEmail: 'foo',
      defaultMessage: 'abc',
      sendProblemReport: spy((_report) => Promise.resolve()),
    });
    const component = shallow(<Support {...props} />);
    const sendButton = component.find({ testName: 'support__send_logs' });

    expect(sendButton.prop('disabled')).to.be.false;
    await click(sendButton);
    expect(props.sendProblemReport).to.have.been.called.once;
  });

  it('should not call send callback when description is empty', () => {
    const props = makeProps({ defaultMessage: '' });
    const component = shallow(<Support {...props} />);
    const sendButton = component.find({ testName: 'support__send_logs' });

    expect(sendButton.prop('disabled')).to.be.true;
  });

  it('should not collect report twice', async () => {
    const props = makeProps({
      collectProblemReport: spy(() => Promise.resolve('/path/to/problem/report')),
    });
    const component = shallow(<Support {...props} />);
    const viewButton = component.find({ testName: 'support__view_logs' });

    await Promise.all([click(viewButton), click(viewButton)]);
    expect(props.collectProblemReport).to.have.been.called.once;
  });

  it('should collect report on submission', async () => {
    const props = makeProps({
      defaultMessage: '',
      defaultEmail: 'foo',
      collectProblemReport: spy(() => Promise.resolve('/path/to/problem/report')),
      sendProblemReport: spy(() => Promise.resolve()),
    });
    const component = shallow(<Support {...props} />);
    const sendButton = component.find({ testName: 'support__send_logs' });

    await click(sendButton);
    expect(props.collectProblemReport).to.have.been.called.once;
    expect(props.sendProblemReport).to.have.been.called.once;
  });

  it('should save the report form on change', () => {
    const props = makeProps({
      defaultEmail: 'email@domain',
      defaultMessage: 'test message',
      sendProblemReport: () => Promise.reject(new Error('Simulation')),
      saveReportForm: spy(),
    });
    const component = shallow(<Support {...props} />);
    const input = component.find({ testName: 'support__form_message' });
    input.simulate('changeText', 'new message');
    expect(props.saveReportForm).to.have.been.called.once;
  });

  it('should clear the report form upon successful submission', async () => {
    const props = makeProps({
      defaultEmail: 'email@domain',
      defaultMessage: 'test message',
      sendProblemReport: () => Promise.resolve(),
      clearReportForm: spy(),
    });
    const component = shallow(<Support {...props} />);
    const sendButton = component.find({ testName: 'support__send_logs' });

    await click(sendButton);
    expect(props.clearReportForm).to.have.been.called.once;
  });
});

function makeProps(mergeProps: $Shape<SupportProps> = {}): SupportProps {
  const defaultProps: SupportProps = {
    defaultEmail: '',
    defaultMessage: '',
    accountHistory: [],
    onClose: () => {},
    viewLog: (_path) => {},
    collectProblemReport: () => Promise.resolve('/path/to/problem/report'),
    sendProblemReport: (_report) => Promise.resolve(),
    saveReportForm: (_form) => {},
    clearReportForm: () => {},
  };
  return { ...defaultProps, ...mergeProps };
}

function click(component) {
  return component.prop('onPress')();
}
