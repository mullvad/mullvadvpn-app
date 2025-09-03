import { ApplicationIcon } from '../../../../../../../application-icon';
import { useApplication, useDisabled } from '../../hooks';

export function Icon() {
  const application = useApplication();
  const disabled = useDisabled();

  return <ApplicationIcon icon={application.icon} disabled={disabled} />;
}
