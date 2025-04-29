import { ConnectionBlocked, ResumeUpgrade } from './components';
import { useShowConnectionBlocked } from './hooks';

export function PauseFooter() {
  const showConnectionBlocked = useShowConnectionBlocked();

  if (showConnectionBlocked) {
    return <ConnectionBlocked />;
  }

  return <ResumeUpgrade />;
}
