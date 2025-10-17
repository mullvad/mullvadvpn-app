import { strings } from '../../../../../../../shared/constants';
import { ObfuscationType } from '../../../../../../../shared/daemon-rpc-types';
import { messages } from '../../../../../../../shared/gettext';

export function getObfuscationLabel(obfuscationType: ObfuscationType): string {
  switch (obfuscationType) {
    case ObfuscationType.auto:
      return messages.gettext('Automatic');
    case ObfuscationType.lwo:
      return strings.lwo;
    case ObfuscationType.quic:
      return strings.quic;
    case ObfuscationType.shadowsocks:
      return messages.pgettext('wireguard-settings-view', 'Shadowsocks');
    case ObfuscationType.udp2tcp:
      return messages.pgettext('wireguard-settings-view', 'UDP-over-TCP');
    case ObfuscationType.off:
      return messages.gettext('Port');
    default:
      return obfuscationType satisfies never;
  }
}
