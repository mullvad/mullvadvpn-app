import { expect } from 'chai';
import { it, describe } from 'mocha';
import RelaySettingsBuilder from '../../src/shared/relay-settings-builder';

describe('Relay settings builder', () => {
  it('should set location to any', () => {
    expect(RelaySettingsBuilder.normal().location.any().build()).to.deep.equal({
      normal: {
        location: 'any',
      },
    });
  });

  it('should bound location to city', () => {
    expect(RelaySettingsBuilder.normal().location.city('se', 'mma').build()).to.deep.equal({
      normal: {
        location: {
          only: {
            country: 'se', city: 'mma',
          },
        },
      },
    });
  });

  it('should bound location to country', () => {
    expect(RelaySettingsBuilder.normal().location.country('se').build()).to.deep.equal({
      normal: {
        location: {
          only: { country: 'se' },
        },
      },
    });
  });

  it('should set openvpn settings to any', () => {
    expect(
      RelaySettingsBuilder.normal()
        .tunnel.openvpn((openvpn) => {
          openvpn.port.any().protocol.any();
        })
        .build(),
    ).to.deep.equal({
      normal: {
        openvpnConstraints: {
          port: 'any',
          protocol: 'any',
        },
      },
    });
  });

  it('should set openvpn settings to exact values', () => {
    expect(
      RelaySettingsBuilder.normal()
        .tunnel.openvpn((openvpn) => {
          openvpn.port.exact(80).protocol.exact('tcp');
        })
        .build(),
    ).to.deep.equal({
      normal: {
        openvpnConstraints: {
          port: { only: 80 },
          protocol: { only: 'tcp' },
        },
      },
    });
  });

  it('should set location from raw RelayLocation', () => {
    expect(RelaySettingsBuilder.normal().location.fromRaw('any').build()).to.deep.equal({
      normal: {
        location: 'any',
      },
    });

    expect(RelaySettingsBuilder.normal().location.fromRaw({ country: 'se' }).build()).to.deep.equal(
      {
        normal: {
          location: {
            only: { country: 'se' },
          },
        },
      },
    );

    expect(
      RelaySettingsBuilder.normal()
        .location.fromRaw({ country: 'se', city: 'mma' })
        .build(),
    ).to.deep.equal({
      normal: {
        location: {
          only: { country: 'se', city: 'mma' },
        },
      },
    });
  });
});
