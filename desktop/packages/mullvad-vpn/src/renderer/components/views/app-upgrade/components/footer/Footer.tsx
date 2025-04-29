import {
  DownloadFooter,
  ErrorFooter,
  InitialFooter,
  LaunchFooter,
  PauseFooter,
  VerifyFooter,
} from './components';
import { useStep } from './hooks';

export function Footer() {
  const step = useStep();

  switch (step) {
    case 'download':
      return <DownloadFooter />;
    case 'error':
      return <ErrorFooter />;
    case 'initial':
      return <InitialFooter />;
    case 'launch':
      return <LaunchFooter />;
    case 'pause':
      return <PauseFooter />;
    case 'verify':
      return <VerifyFooter />;
    default:
      return step satisfies never;
  }
}
