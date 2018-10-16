// @flow

import * as React from 'react';
import { shallow } from 'enzyme';
import { CloseBarItem } from '../../src/renderer/components/NavigationBar';
import SelectLocation from '../../src/renderer/components/SelectLocation';

describe('components/SelectLocation', () => {
  const defaultProps = {
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
                includeInCountry: true,
                weight: 1,
              },
              {
                hostname: 'fake2.mullvad.net',
                ipv4AddrIn: '192.168.0.101',
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
                includeInCountry: true,
                weight: 1,
              },
            ],
          },
        ],
      },
    ],
  };

  it('should call close callback', (done) => {
    const props = {
      ...defaultProps,
      onClose: () => done(),
      onSelect: () => {},
    };
    const component = shallow(<SelectLocation {...props} />)
      .find(CloseBarItem)
      .dive();
    component.simulate('press');
  });

  it('should call select callback for country', (done) => {
    const props = {
      ...defaultProps,
      onClose: () => {},
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
    };
    const component = shallow(<SelectLocation {...props} />);
    const elements = getComponent(component, 'country');
    expect(elements).to.have.length(1);
    elements.at(0).simulate('press');
  });

  it('should call select callback for city', (done) => {
    const props = {
      ...defaultProps,
      onClose: () => {},
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
    };
    const component = shallow(<SelectLocation {...props} />);
    const elements = getComponent(component, 'city');
    expect(elements).to.have.length(2);
    elements.at(0).simulate('press');
  });

  it('should call select callback for relay', (done) => {
    const props = {
      ...defaultProps,
      onClose: () => {},
      onSelect: (location) => {
        try {
          expect(location).to.deep.equal({
            hostname: ['se', 'mma', 'fake1.mullvad.net'],
          });
          done();
        } catch (e) {
          done(e);
        }
      },
    };
    const component = shallow(<SelectLocation {...props} />);
    const elements = getComponent(component, 'relay');
    expect(elements).to.have.length(2);
    elements.at(0).simulate('press');
  });
});

function getComponent(container, testName) {
  return container.findWhere((n) => n.prop('testName') === testName);
}
