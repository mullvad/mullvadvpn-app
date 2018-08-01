// @flow

import * as React from 'react';
import { shallow } from 'enzyme';
import Settings from '../../app/components/Settings';
import { CloseBarItem } from '../../app/components/NavigationBar';

type SettingsProps = React.ElementProps<typeof Settings>;

describe('components/Settings', () => {
  const makeProps = (mergeProps: $Shape<SettingsProps> = {}): SettingsProps => {
    const defaultProps: SettingsProps = {
      loginState: 'none',
      accountExpiry: null,
      appVersion: '',
      onQuit: () => {},
      onClose: () => {},
      onViewAccount: () => {},
      onViewSupport: () => {},
      onViewAdvancedSettings: () => {},
      onViewPreferences: () => {},
      onExternalLink: (_type) => {},
      updateAccountExpiry: () => Promise.resolve(),
    };
    return Object.assign({}, defaultProps, mergeProps);
  };

  it('should show quit button when logged out', () => {
    const props = makeProps({
      loginState: 'none',
      accountExpiry: null,
    });
    const component = getComponent(render(props), 'settings__quit');
    expect(component).to.have.length(1);
  });

  it('should show quit button when logged in', () => {
    const props = makeProps({
      accountExpiry: new Date('2038-01-01').toISOString(),
      loginState: 'ok',
    });
    const component = getComponent(render(props), 'settings__quit');
    expect(component).to.have.length(1);
  });

  it('should show external links when logged out', () => {
    const props = makeProps({
      loginState: 'none',
      accountExpiry: null,
    });
    const component = getComponent(render(props), 'settings__external_link');
    expect(component.length).to.be.above(0);
  });

  it('should show external links when logged in', () => {
    const props = makeProps({
      accountExpiry: new Date('2038-01-01').toISOString(),
      loginState: 'ok',
    });
    const component = getComponent(render(props), 'settings__external_link');
    expect(component.length).to.be.above(0);
  });

  it('should show account section when logged in', () => {
    const props = makeProps({
      accountExpiry: new Date('2038-01-01').toISOString(),
      loginState: 'ok',
    });
    const component = getComponent(render(props), 'settings__account');
    expect(component).to.have.length(1);
  });

  it('should hide account section when logged out', () => {
    const props = makeProps({
      loginState: 'none',
      accountExpiry: null,
    });
    const elements = getComponent(render(props), 'settings__account');
    expect(elements).to.have.length(0);
  });

  it('should hide account link when not logged in', () => {
    const props = makeProps({
      loginState: 'none',
      accountExpiry: null,
    });
    const elements = getComponent(render(props), 'settings__view_account');
    expect(elements).to.have.length(0);
  });

  it('should show out-of-time message for unpaid account', () => {
    const props = makeProps({
      accountExpiry: new Date('2001-01-01').toISOString(),
      loginState: 'ok',
    });
    const component = getComponent(render(props), 'settings__account_paid_until_subtext');
    expect(component.children().text()).to.equal('OUT OF TIME');
  });

  it('should hide out-of-time message for paid account', () => {
    const props = makeProps({
      accountExpiry: new Date('2038-01-01').toISOString(),
      loginState: 'ok',
    });
    const component = getComponent(render(props), 'settings__account_paid_until_subtext');
    expect(component.children().text()).not.to.equal('OUT OF TIME');
  });

  it('should call close callback', (done) => {
    const props = makeProps({
      onClose: () => done(),
      loginState: 'none',
      accountExpiry: null,
    });
    const component = render(props)
      .find(CloseBarItem)
      .dive();
    component.simulate('press');
  });

  it('should call quit callback', (done) => {
    const props = makeProps({
      onQuit: () => done(),

      loginState: 'none',
      accountExpiry: null,
    });
    const component = getComponent(render(props), 'settings__quit');
    component.simulate('press');
  });

  it('should call account callback', (done) => {
    const props = makeProps({
      onViewAccount: () => done(),
      accountExpiry: new Date('2038-01-01').toISOString(),
      loginState: 'ok',
    });
    const component = getComponent(render(props), 'settings__account_paid_until_button');
    component.simulate('press');
  });

  it('should call advanced settings callback', (done) => {
    const props = makeProps({
      onViewAdvancedSettings: () => done(),
      accountExpiry: new Date('2038-01-01').toISOString(),
      loginState: 'ok',
    });
    const component = getComponent(render(props), 'settings__advanced');
    component.simulate('press');
  });

  it('should call preferences callback', (done) => {
    const props = makeProps({
      onViewPreferences: () => done(),
      accountExpiry: new Date('2038-01-01').toISOString(),
      loginState: 'ok',
    });
    const component = getComponent(render(props), 'settings__preferences');
    component.simulate('press');
  });

  it('should call support callback', (done) => {
    const props = makeProps({
      onViewSupport: () => done(),
      accountExpiry: new Date('2038-01-01').toISOString(),
      loginState: 'ok',
    });
    const component = getComponent(render(props), 'settings__view_support');
    component.simulate('press');
  });

  it('should call external links callback', () => {
    const collectedExternalLinkTypes: Array<string> = [];
    const props = makeProps({
      onExternalLink: (type) => {
        collectedExternalLinkTypes.push(type);
      },

      loginState: 'none',
      accountExpiry: null,
    });
    const container = getComponent(render(props), 'settings__external_link');
    container
      .find({ testName: 'settings__external_link' })
      .forEach((element) => element.simulate('press'));

    expect(collectedExternalLinkTypes).to.include.ordered.members(['faq', 'guides']);
  });
});

function render(props) {
  return shallow(<Settings {...props} />);
}

function getComponent(container, testName) {
  return container.findWhere((n) => n.prop('testName') === testName);
}
