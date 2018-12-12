// @flow

import * as React from 'react';
import { shallow } from 'enzyme';
import Account from '../../src/renderer/components/Account';
import { BackBarItem } from '../../src/renderer/components/NavigationBar';

type AccountProps = React.ElementProps<typeof Account>;

describe('components/Account', () => {
  const makeProps = (mergeProps: $Shape<AccountProps>): AccountProps => {
    const defaultProps: AccountProps = {
      accountToken: '1234',
      accountExpiry: new Date('2038-01-01').toISOString(),
      expiryLocale: 'en-US',
      isOffline: false,
      updateAccountExpiry: () => Promise.resolve(),
      onClose: () => {},
      onLogout: () => {},
      onBuyMore: () => {},
    };
    return {
      ...defaultProps,
      ...mergeProps,
    };
  };

  it('should call close callback', (done) => {
    const props = makeProps({
      onClose: () => done(),
    });
    const component = render(props)
      .find(BackBarItem)
      .dive();
    component.simulate('press');
  });

  it('should call logout callback', (done) => {
    const props = makeProps({
      onLogout: () => done(),
    });
    const component = getComponent(render(props), 'account__logout');
    component.simulate('press');
  });

  it('should call "buy more" callback', (done) => {
    const props = makeProps({
      onBuyMore: () => done(),
    });
    const component = getComponent(render(props), 'account__buymore');
    component.simulate('press');
  });

  it('should display "out of time" message when account expired', () => {
    const props = makeProps({
      accountExpiry: new Date('2001-01-01').toISOString(),
    });
    const component = getComponent(render(props), 'account__expiry');
    expect(component.dive().html()).to.contain('OUT OF TIME');
  });

  it('should not display "out of time" message when account is active', () => {
    const props = makeProps({});
    const component = getComponent(render(props), 'account__expiry');
    expect(component.dive().html()).to.not.contain('OUT OF TIME');
  });

  it('should not display "unavailable" message when account expiry is missing', () => {
    const props = makeProps({
      accountExpiry: null,
    });
    const component = getComponent(render(props), 'account__expiry');
    expect(component.dive().html()).to.contain('Currently unavailable');
  });
});

function render(props) {
  return shallow(<Account {...props} />);
}

function getComponent(container, testName) {
  return container.findWhere((n) => n.prop('testName') === testName);
}
