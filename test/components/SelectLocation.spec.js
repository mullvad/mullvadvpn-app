// @flow

import { expect } from 'chai';
import * as React from 'react';
import ReactTestUtils, { Simulate } from 'react-dom/test-utils';
import SelectLocation from '../../app/components/SelectLocation';

import type { SettingsReduxState } from '../../app/redux/settings/reducers';
import type { SelectLocationProps } from '../../app/components/SelectLocation';

describe('components/SelectLocation', () => {
  const state: SettingsReduxState = {
    relaySettings: {
      normal: {
        location: 'any',
        protocol: 'any',
        port: 'any',
      }
    },
    relayLocations: [{
      name: 'Sweden',
      code: 'se',
      hasActiveRelays: true,
      cities: [{
        name: 'Malmö',
        code: 'mma',
        latitude: 0,
        longitude: 0,
        hasActiveRelays: true,
      }, {
        name: 'Stockholm',
        code: 'sto',
        latitude: 0,
        longitude: 0,
        hasActiveRelays: true,
      }],
    }],
    allowLan: false,
  };

  const makeProps = (state: SettingsReduxState, mergeProps: $Shape<SelectLocationProps>): SelectLocationProps => {
    const defaultProps: SelectLocationProps = {
      settings: state,
      onClose: () => {},
      onSelect: (_server) => {}
    };
    return Object.assign({}, defaultProps, mergeProps);
  };

  const render = (props: SelectLocationProps) => {
    return ReactTestUtils.renderIntoDocument(
      <SelectLocation { ...props } />
    );
  };

  it('should call close callback', (done) => {
    const props = makeProps(state, {
      onClose: () => done()
    });
    const domNode = ReactTestUtils.findRenderedDOMComponentWithClass(render(props), 'select-location__close');
    // See: https://github.com/facebook/flow/pull/5841
    if(domNode) {
      Simulate.click(domNode);
    }
  });

  it('should call select callback for country', (done) => {
    const props = makeProps(state, {
      onSelect: (location) => {
        try {
          expect(location).to.deep.equal({
            country: 'se'
          });
          done();
        } catch(e) {
          done(e);
        }
      }
    });
    const elements = ReactTestUtils.scryRenderedDOMComponentsWithClass(render(props), 'select-location__cell');
    expect(elements).to.have.length(1);
    Simulate.click(elements[0]);
  });

  it('should call select callback for city', (done) => {
    const props = makeProps(state, {
      onSelect: (location) => {
        try {
          expect(location).to.deep.equal({
            city: ['se', 'mma']
          });
          done();
        } catch(e) {
          done(e);
        }
      }
    });
    const elements = ReactTestUtils.scryRenderedDOMComponentsWithClass(render(props), 'select-location__sub-cell');
    expect(elements).to.have.length(2);
    Simulate.click(elements[0]);
  });

});
