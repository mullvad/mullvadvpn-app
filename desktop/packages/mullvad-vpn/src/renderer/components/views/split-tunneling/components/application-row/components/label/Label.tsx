import { ApplicationLabel } from '../../../application-label';
import { useApplication } from '../../hooks';

export function Label() {
  const application = useApplication();

  return <ApplicationLabel>{application.name}</ApplicationLabel>;
}
