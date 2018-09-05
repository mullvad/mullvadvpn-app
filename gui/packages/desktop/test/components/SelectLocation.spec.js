// @flow

import * as React from 'react';
import { shallow } from 'enzyme';
import { CloseBarItem } from '../../src/renderer/components/NavigationBar';
import SelectLocation from '../../src/renderer/components/SelectLocation';

import type { SettingsReduxState } from '../../src/renderer/redux/settings/reducers';
import type { SelectLocationProps } from '../../src/renderer/components/SelectLocation';

describe('components/SelectLocation', () => {
  const state: SettingsReduxState = {
    relaySettings: {
      normal: {
        location: 'any',
        protocol: 'any',
        port: 'any',
      },
    },
    relayLocations: [
      {
        name: 'Sweden',
        code: 'se',
        hasActiveRelays: true,
        cities: [
          {
            name: 'Malmö',
            code: 'mma',
            latitude: 0,
            longitude: 0,
            hasActiveRelays: true,
            relays: [
              {
                hostname: 'fake1.mullvad.net',
                ipv4AddrIn: '192.168.0.100',
                ipv4AddrExit: '192.168.1.100',
                includeInCountry: true,
                weight: 1,
              },
            ],
          },
          {
            name: 'Stockholm',
            code: 'sto',
            latitude: 0,
            longitude: 0,
            hasActiveRelays: true,
            relays: [
              {
                hostname: 'fake2.mullvad.net',
                ipv4AddrIn: '192.168.0.101',
                ipv4AddrExit: '192.168.1.101',
                includeInCountry: true,
                weight: 1,
              },
            ],
          },
        ],
      },
    ],
    autoConnect: false,
    allowLan: false,
    enableIpv6: true,
  };

  const makeProps = (
    state: SettingsReduxState,
    mergeProps: $Shape<SelectLocationProps>,
  ): SelectLocationProps => {
    const defaultProps: SelectLocationProps = {
      settings: state,
      onClose: () => {},
      onSelect: (_server) => {},
    };
    return Object.assign({}, defaultProps, mergeProps);
  };

  const render = (props: SelectLocationProps) => {
    return shallow(<SelectLocation {...props} />);
  };

  it('should call close callback', (done) => {
    const props = makeProps(state, {
      onClose: () => done(),
    });
    const component = render(props)
      .find(CloseBarItem)
      .dive();
    component.simulate('press');
  });

  it('should call select callback for country', (done) => {
    const props = makeProps(state, {
      onSelect: (location) => {
        try {
          expect(location).to.deep.equal({
            country: 'se',
          });
          done();
        } catch (e) {
          done(e);
        }
      },
    });
    const elements = getComponent(render(props), 'country');
    expect(elements).to.have.length(1);
    elements.at(0).simulate('press');
  });

  it('should call select callback for city', (done) => {
    const props = makeProps(state, {
      onSelect: (location) => {
        try {
          expect(location).to.deep.equal({
            city: ['se', 'mma'],
          });
          done();
        } catch (e) {
          done(e);
        }
      },
    });
    const elements = getComponent(render(props), 'city');
    expect(elements).to.have.length(2);
    elements.at(0).simulate('press');
  });
});

function getComponent(container, testName) {
  return container.findWhere((n) => n.prop('testName') === testName);
}
