// @flow

import * as React from 'react';
import { shallow } from 'enzyme';
import Settings from '../../src/renderer/components/Settings';
import { CloseBarItem } from '../../src/renderer/components/NavigationBar';

type SettingsProps = React.ElementProps<typeof Settings>;

describe('components/Settings', () => {
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

  it('should show quit button when logged out', () => {
    const props = {
      ...defaultProps,
      loginState: 'none',
      accountExpiry: null,
    };
    const component = getComponent(shallow(<Settings {...props} />), 'settings__quit');
    expect(component).to.have.length(1);
  });

  it('should show quit button when logged in', () => {
    const props = {
      ...defaultProps,
      loginState: 'ok',
      accountExpiry: new Date('2038-01-01').toISOString(),
    };
    const component = getComponent(shallow(<Settings {...props} />), 'settings__quit');
    expect(component).to.have.length(1);
  });

  it('should show external links when logged out', () => {
    const props = {
      ...defaultProps,
      loginState: 'none',
      accountExpiry: null,
    };
    const component = getComponent(shallow(<Settings {...props} />), 'settings__external_link');
    expect(component.length).to.be.above(0);
  });

  it('should show external links when logged in', () => {
    const props = {
      ...defaultProps,
      loginState: 'ok',
      accountExpiry: new Date('2038-01-01').toISOString(),
    };
    const component = getComponent(shallow(<Settings {...props} />), 'settings__external_link');
    expect(component.length).to.be.above(0);
  });

  it('should show account section when logged in', () => {
    const props = {
      ...defaultProps,
      loginState: 'ok',
      accountExpiry: new Date('2038-01-01').toISOString(),
    };
    const component = getComponent(shallow(<Settings {...props} />), 'settings__account');
    expect(component).to.have.length(1);
  });

  it('should hide account section when logged out', () => {
    const props = {
      ...defaultProps,
      loginState: 'none',
      accountExpiry: null,
    };
    const elements = getComponent(shallow(<Settings {...props} />), 'settings__account');
    expect(elements).to.have.length(0);
  });

  it('should hide account link when not logged in', () => {
    const props = {
      ...defaultProps,
      loginState: 'none',
      accountExpiry: null,
    };
    const elements = getComponent(shallow(<Settings {...props} />), 'settings__view_account');
    expect(elements).to.have.length(0);
  });

  it('should show out-of-time message for unpaid account', () => {
    const props = {
      ...defaultProps,
      loginState: 'ok',
      accountExpiry: new Date('2001-01-01').toISOString(),
    };
    const component = getComponent(
      shallow(<Settings {...props} />),
      'settings__account_paid_until_subtext',
    );
    expect(component.children().text()).to.equal('OUT OF TIME');
  });

  it('should hide out-of-time message for paid account', () => {
    const props = {
      ...defaultProps,
      loginState: 'ok',
      accountExpiry: new Date('2038-01-01').toISOString(),
    };
    const component = getComponent(
      shallow(<Settings {...props} />),
      'settings__account_paid_until_subtext',
    );
    expect(component.children().text()).not.to.equal('OUT OF TIME');
  });

  it('should call close callback', (done) => {
    const props = {
      ...defaultProps,
      loginState: 'none',
      accountExpiry: null,
      onClose: () => done(),
    };
    const component = shallow(<Settings {...props} />)
      .find(CloseBarItem)
      .dive();
    component.simulate('press');
  });

  it('should call quit callback', (done) => {
    const props = {
      ...defaultProps,
      loginState: 'none',
      accountExpiry: null,
      onQuit: () => done(),
    };
    const component = getComponent(shallow(<Settings {...props} />), 'settings__quit');
    component.simulate('press');
  });

  it('should call account callback', (done) => {
    const props = {
      ...defaultProps,
      loginState: 'ok',
      accountExpiry: new Date('2038-01-01').toISOString(),
      onViewAccount: () => done(),
    };
    const component = getComponent(
      shallow(<Settings {...props} />),
      'settings__account_paid_until_button',
    );
    component.simulate('press');
  });

  it('should call advanced settings callback', (done) => {
    const props = {
      ...defaultProps,
      loginState: 'ok',
      accountExpiry: new Date('2038-01-01').toISOString(),
      onViewAdvancedSettings: () => done(),
    };
    const component = getComponent(shallow(<Settings {...props} />), 'settings__advanced');
    component.simulate('press');
  });

  it('should call preferences callback', (done) => {
    const props = {
      ...defaultProps,
      loginState: 'ok',
      accountExpiry: new Date('2038-01-01').toISOString(),
      onViewPreferences: () => done(),
    };
    const component = getComponent(shallow(<Settings {...props} />), 'settings__preferences');
    component.simulate('press');
  });

  it('should call support callback', (done) => {
    const props = {
      ...defaultProps,
      loginState: 'ok',
      accountExpiry: new Date('2038-01-01').toISOString(),
      onViewSupport: () => done(),
    };
    const component = getComponent(shallow(<Settings {...props} />), 'settings__view_support');
    component.simulate('press');
  });

  it('should call external links callback', () => {
    const collectedExternalLinkTypes: Array<string> = [];
    const props = {
      ...defaultProps,
      loginState: 'none',
      accountExpiry: null,
      onExternalLink: (type) => {
        collectedExternalLinkTypes.push(type);
      },
    };
    const container = getComponent(shallow(<Settings {...props} />), 'settings__external_link');
    container
      .find({ testName: 'settings__external_link' })
      .forEach((element) => element.simulate('press'));

    expect(collectedExternalLinkTypes).to.include.ordered.members(['faq', 'guides']);
  });
});

function getComponent(container, testName) {
  return container.findWhere((n) => n.prop('testName') === testName);
}
