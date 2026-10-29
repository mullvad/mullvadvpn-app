import React from 'react';

import { FeatureIndicator } from '../../../../../../../../../shared/daemon-rpc-types';
import {
  CustomDnsFeatureIndicator,
  DaitaFeatureIndicator,
  DnsContentBlockersFeatureIndicator,
  LanSharingFeatureIndicator,
  LockdownModeFeatureIndicator,
  LwoFeatureIndicator,
  MagicMultihopFeatureIndicator,
  MtuFeatureIndicator,
  MultihopFeatureIndicator,
  QuantumResistanceFeatureIndicator,
  QuicFeatureIndicator,
  ServerIpOverrideFeatureIndicator,
  ShadowsocksFeatureIndicator,
  SplitTunnelingFeatureIndicator,
  UdpOverTcpFeatureIndicator,
  WireguardPortFeatureIndicator,
} from '../components';

export const useGetFeatureIndicatorMap = () => {
  const featureIndicatorMap: Record<FeatureIndicator, React.ReactNode> = {
    [FeatureIndicator.daita]: <DaitaFeatureIndicator />,
    [FeatureIndicator.multihopAuto]: <MagicMultihopFeatureIndicator />,
    [FeatureIndicator.udp2tcp]: <UdpOverTcpFeatureIndicator />,
    [FeatureIndicator.shadowsocks]: <ShadowsocksFeatureIndicator />,
    [FeatureIndicator.quic]: <QuicFeatureIndicator />,
    [FeatureIndicator.lwo]: <LwoFeatureIndicator />,
    [FeatureIndicator.wireGuardPort]: <WireguardPortFeatureIndicator />,
    [FeatureIndicator.multihop]: <MultihopFeatureIndicator />,
    [FeatureIndicator.customDns]: <CustomDnsFeatureIndicator />,
    [FeatureIndicator.customMtu]: <MtuFeatureIndicator />,
    [FeatureIndicator.lanSharing]: <LanSharingFeatureIndicator />,
    [FeatureIndicator.lockdownMode]: <LockdownModeFeatureIndicator />,
    [FeatureIndicator.splitTunneling]: <SplitTunnelingFeatureIndicator />,
    [FeatureIndicator.serverIpOverride]: <ServerIpOverrideFeatureIndicator />,
    [FeatureIndicator.quantumResistance]: <QuantumResistanceFeatureIndicator />,
    [FeatureIndicator.dnsContentBlockers]: <DnsContentBlockersFeatureIndicator />,
  };

  return featureIndicatorMap;
};
