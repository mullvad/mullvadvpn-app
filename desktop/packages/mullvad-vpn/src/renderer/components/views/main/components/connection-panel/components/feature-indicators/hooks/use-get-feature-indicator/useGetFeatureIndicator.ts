import React from 'react';
import { sprintf } from 'sprintf-js';

import { strings } from '../../../../../../../../../../shared/constants';
import { FeatureIndicator } from '../../../../../../../../../../shared/daemon-rpc-types';
import { messages } from '../../../../../../../../../../shared/gettext';
import { RoutePath } from '../../../../../../../../../../shared/routes';
import { TransitionType, useHistory } from '../../../../../../../../../lib/history';

export const useGetFeatureIndicator = () => {
  const history = useHistory();

  const gotoDaitaFeature = React.useCallback(() => {
    history.push(RoutePath.daitaSettings, {
      transition: TransitionType.show,
      options: [
        {
          type: 'scroll-to-anchor',
          id: 'daita-enable-setting',
        },
      ],
    });
  }, [history]);

  const gotoMultihopFeature = React.useCallback(() => {
    history.push(RoutePath.multihopSettings, {
      transition: TransitionType.show,
    });
  }, [history]);

  const gotoCustomDnsFeature = React.useCallback(() => {
    history.push(RoutePath.vpnSettings, {
      transition: TransitionType.show,
      options: [
        {
          type: 'scroll-to-anchor',
          id: 'custom-dns-settings',
        },
      ],
    });
  }, [history]);

  const gotoLanSharingFeature = React.useCallback(() => {
    history.push(RoutePath.vpnSettings, {
      transition: TransitionType.show,
      options: [
        {
          type: 'scroll-to-anchor',
          id: 'allow-lan-setting',
        },
      ],
    });
  }, [history]);

  const gotoLockdownModeFeature = React.useCallback(() => {
    history.push(RoutePath.vpnSettings, {
      transition: TransitionType.show,
      options: [
        {
          type: 'scroll-to-anchor',
          id: 'lockdown-mode-setting',
        },
      ],
    });
  }, [history]);

  const gotoSplitTunnelingFeature = React.useCallback(() => {
    history.push(RoutePath.splitTunneling, {
      transition: TransitionType.show,
    });
  }, [history]);

  const gotoDnsContentBlockersFeature = React.useCallback(() => {
    history.push(RoutePath.vpnSettings, {
      transition: TransitionType.show,
      options: [
        {
          type: 'scroll-to-anchor',
          id: 'dns-blocker-setting',
        },
      ],
    });
  }, [history]);

  const featureMap: Record<FeatureIndicator, { label: string; onClick?: () => void }> = {
    [FeatureIndicator.daita]: { label: strings.daita, onClick: gotoDaitaFeature },
    [FeatureIndicator.daitaMultihop]: {
      label: sprintf(
        // TRANSLATORS: This is used as a feature indicator to show that DAITA is enabled through
        // TRANSLATORS: multihop.
        // TRANSLATORS: Available placeholders:
        // TRANSLATORS: %(DAITA)s - Is a non-translatable feature "DAITA"
        messages.pgettext('connect-view', '%(DAITA)s: Multihop'),
        {
          DAITA: strings.daita,
        },
      ),
    },
    [FeatureIndicator.udp2tcp]: {
      label: messages.pgettext('wireguard-settings-view', 'Obfuscation'),
    },
    [FeatureIndicator.shadowsocks]: {
      label: messages.pgettext('wireguard-settings-view', 'Obfuscation'),
    },
    [FeatureIndicator.quic]: { label: messages.pgettext('wireguard-settings-view', 'Obfuscation') },
    [FeatureIndicator.multihop]: {
      label:
        // TRANSLATORS: This refers to the multihop setting in the VPN settings view. This is
        // TRANSLATORS: displayed when the feature is on.
        messages.gettext('Multihop'),
      onClick: gotoMultihopFeature,
    },
    [FeatureIndicator.customDns]: {
      label:
        // TRANSLATORS: This refers to the Custom DNS setting in the VPN settings view. This is
        // TRANSLATORS: displayed when the feature is on.
        messages.gettext('Custom DNS'),
      onClick: gotoCustomDnsFeature,
    },
    [FeatureIndicator.customMtu]: { label: messages.pgettext('wireguard-settings-view', 'MTU') },
    [FeatureIndicator.bridgeMode]: {
      label: messages.pgettext('openvpn-settings-view', 'Bridge mode'),
    },
    [FeatureIndicator.lanSharing]: {
      label: messages.pgettext('vpn-settings-view', 'Local network sharing'),
      onClick: gotoLanSharingFeature,
    },
    [FeatureIndicator.customMssFix]: {
      label: messages.pgettext('openvpn-settings-view', 'Mssfix'),
    },
    [FeatureIndicator.lockdownMode]: {
      label: messages.pgettext('vpn-settings-view', 'Lockdown mode'),
      onClick: gotoLockdownModeFeature,
    },
    [FeatureIndicator.splitTunneling]: {
      label: strings.splitTunneling,
      onClick: gotoSplitTunnelingFeature,
    },
    [FeatureIndicator.serverIpOverride]: {
      label: messages.pgettext('settings-import', 'Server IP override'),
    },
    [FeatureIndicator.quantumResistance]:
      // TRANSLATORS: This refers to the quantum resistance setting in the WireGuard settings view.
      // TRANSLATORS: This is displayed when the feature is on.
      { label: messages.gettext('Quantum resistance') },
    [FeatureIndicator.dnsContentBlockers]: {
      label: messages.pgettext('vpn-settings-view', 'DNS content blockers'),
      onClick: gotoDnsContentBlockersFeature,
    },
  };

  return featureMap;
};
