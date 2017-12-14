// @flow

import { expect } from 'chai';
import React from 'react';
import { shallow } from 'enzyme';
import HeaderBar from '../../app/components/HeaderBar';

require('../setup/enzyme');

describe('components/HeaderBar', () => {

  it('should display headerbar', () => {
    const component = render({
      hidden: false,
    });
    const hasChildMatching = hasChild(component, 'headerbar__container');
    expect(hasChildMatching).to.be.true;
  });

  it('should not display headerbar', () => {
    const component = render({
      hidden: true,
    });
    const hasChildMatching = hasChild(component, 'headerbar__container');
    expect(hasChildMatching).to.be.false;
  });

  it('should display settings button', () => {
    const component = render({
      showSettings: true,
    });
    const hasChildMatching = hasChild(component, 'headerbar__settings');
    expect(hasChildMatching).to.be.true;
  });

  it('should not display settings button', () => {
    const component = render({
      showSettings: false,
    });
    const hasChildMatching = hasChild(component, 'headerbar__settings');
    expect(hasChildMatching).to.be.false;
  });

  it('should call settings callback', (done) => {
    const component = render({
      showSettings: true,
      onSettings:  () => done(),
    });
    const settingsButton = getComponent(component, 'headerbar__settings');
    click(settingsButton);
  });

});

function render(props) {
  return shallow(
    <HeaderBar {...props} />
  );
}

function getComponent(container, testName) {
  return container.findWhere( n => n.prop('testName') === testName);
}

function hasChild(container, testName) {
  return getComponent(container, testName).length > 0;
}

function click(component) {
  component.prop('onPress')();
}
