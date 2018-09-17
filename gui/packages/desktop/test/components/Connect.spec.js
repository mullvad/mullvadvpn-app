// @flow

import * as React from 'react';
import { shallow } from 'enzyme';

import { TunnelBanner } from '../../src/renderer/components/Connect';

describe('components/Connect', () => {
  describe('TunnelBanner', () => {
    it('invisible when disconnecting', () => {
      for (const reason of ['nothing', 'block', 'reconnect']) {
        const component = shallow(
          <TunnelBanner
            tunnelState={{
              state: 'disconnecting',
              details: { reason },
            }}
          />,
        );
        expect(component.state('visible')).to.be.false;
      }
    });

    it('invisible when connected or disconnected', () => {
      for (const state of ['connected', 'disconnected']) {
        const component = shallow(
          <TunnelBanner
            tunnelState={{
              state,
            }}
          />,
        );
        expect(component.state('visible')).to.be.false;
      }
    });

    it('visible when connecting', () => {
      const component = shallow(
        <TunnelBanner
          tunnelState={{
            state: 'connecting',
          }}
        />,
      );

      expect(component.state('visible')).to.be.true;
      expect(component.state('title')).to.not.be.empty;
    });

    it('visible when blocked', () => {
      const component = shallow(
        <TunnelBanner
          tunnelState={{
            state: 'blocked',
            details: {
              reason: 'no_matching_relay',
            },
          }}
        />,
      );

      expect(component.state('visible')).to.be.true;
      expect(component.state('title')).to.not.be.empty;
      expect(component.state('subtitle')).to.not.be.empty;
    });
  });
});
