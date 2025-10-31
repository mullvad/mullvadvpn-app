import { LiftedConstraint } from '../../../../../shared/daemon-rpc-types';
import { messages } from '../../../../../shared/gettext';

export function formatRelayPort(port: LiftedConstraint<number>): string {
  return port === 'any' ? messages.gettext('Automatic') : `${port}`;
}
