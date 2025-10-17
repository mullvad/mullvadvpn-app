import { Constraint } from '../../../../../shared/daemon-rpc-types';
import { messages } from '../../../../../shared/gettext';

export function formatObfuscationPort(port: Constraint<number>): string {
  return port === 'any' ? messages.gettext('Automatic') : `${port.only}`;
}
