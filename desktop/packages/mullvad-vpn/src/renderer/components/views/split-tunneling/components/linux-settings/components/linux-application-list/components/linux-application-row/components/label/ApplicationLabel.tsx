import { ApplicationLabel } from '../../../../../../../application-label';
import { useApplication, useDisabled } from '../../hooks';

export function Label() {
  const application = useApplication();
  const disabled = useDisabled();

  return <ApplicationLabel disabled={disabled}>{application.name}</ApplicationLabel>;
}
