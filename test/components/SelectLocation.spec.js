// @flow

import { expect } from 'chai';
import * as React from 'react';
import { shallow } from 'enzyme';
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
    return shallow(
      <SelectLocation { ...props } />
    );
  };

  it('should call close callback', (done) => {
    const props = makeProps(state, {
      onClose: () => done()
    });
    const node = getComponent(render(props), 'close');
    click(node);
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
    const elements = getComponent(render(props), 'country');
    expect(elements).to.have.length(1);
    click(elements.at(0));
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
    const elements = getComponent(render(props), 'city');
    expect(elements).to.have.length(2);
    click(elements.at(0));
  });

});

function getComponent(container, testName) {
  return container.findWhere( n => n.prop('testName') === testName);
}

function click(component) {
  component.prop('onPress')();
}
