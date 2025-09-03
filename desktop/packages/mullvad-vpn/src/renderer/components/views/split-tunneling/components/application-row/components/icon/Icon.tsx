import { ApplicationIcon } from '../../../application-icon';
import { useApplication } from '../../hooks';

export function Icon() {
  const application = useApplication();

  return <ApplicationIcon icon={application.icon} />;
}
