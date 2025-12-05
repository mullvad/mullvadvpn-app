import { Constraint, liftConstraint } from '../../../../../shared/daemon-rpc-types';
import { messages } from '../../../../../shared/gettext';

export function formatObfuscationPort(port: Constraint<number>): string {
  const portLiftedConstraint = liftConstraint(port);

  return portLiftedConstraint === 'any' ? messages.gettext('Automatic') : `${portLiftedConstraint}`;
}
