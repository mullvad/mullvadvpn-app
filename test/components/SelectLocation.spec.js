// @flow

import { expect } from 'chai';
import React from 'react';
import ReactTestUtils, { Simulate } from 'react-dom/test-utils';
import SelectLocation from '../../app/components/SelectLocation';

import type { SettingsReduxState } from '../../app/redux/settings/reducers';
import type { SelectLocationProps } from '../../app/components/SelectLocation';

describe('components/SelectLocation', () => {
  const state: SettingsReduxState = {
    relaySettings: {
      host: 'example.com',
      protocol: 'udp',
      port: 1301,
    },
  };

  const makeProps = (state: SettingsReduxState, mergeProps: $Shape<SelectLocationProps>): SelectLocationProps => {
    const defaultProps: SelectLocationProps = {
      settings: state,
      onClose: () => {},
      onSelect: (_server) => {}
    };
    return Object.assign({}, defaultProps, mergeProps);
  };

  const render = (props: SelectLocationProps): SelectLocation => {
    return ReactTestUtils.renderIntoDocument(
      <SelectLocation { ...props } />
    );
  };

  it('should call close callback', (done) => {
    const props = makeProps(state, {
      onClose: () => done()
    });
    const domNode = ReactTestUtils.findRenderedDOMComponentWithClass(render(props), 'select-location__close');
    Simulate.click(domNode);
  });

  it('should call select callback', (done) => {
    const props = makeProps(state, {
      onSelect: (_server) => done()
    });
    const elements = ReactTestUtils.scryRenderedDOMComponentsWithClass(render(props), 'select-location__cell');
    expect(elements).to.have.length.greaterThan(1);
    Simulate.click(elements[1]);
  });

});
