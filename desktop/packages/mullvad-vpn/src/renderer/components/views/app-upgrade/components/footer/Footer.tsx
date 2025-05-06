import styled from 'styled-components';

import { useMeasure } from '../../../../../hooks';
import {
  DownloadFooter,
  ErrorFooter,
  InitialFooter,
  LaunchFooter,
  PauseFooter,
  VerifyFooter,
} from './components';
import { useStep } from './hooks';

const TransitionHeight = styled.div<{ $height: string }>`
  display: flex;
  flex-direction: column;
  justify-content: flex-end;
  overflow: hidden;
  height: ${({ $height }) => $height};
  @media (prefers-reduced-motion: no-preference) {
    transition: height 200ms cubic-bezier(0.16, 1, 0.3, 1);
  }
`;

export function Footer() {
  const step = useStep();
  const [ref, { height }] = useMeasure<HTMLDivElement>();
  let footer: React.ReactElement | null = null;

  switch (step) {
    case 'download':
      footer = <DownloadFooter />;
      break;
    case 'error':
      footer = <ErrorFooter />;
      break;
    case 'initial':
      footer = <InitialFooter />;
      break;
    case 'launch':
      footer = <LaunchFooter />;
      break;
    case 'pause':
      footer = <PauseFooter />;
      break;
    case 'verify':
      footer = <VerifyFooter />;
      break;
    default:
      return step satisfies never;
  }

  return (
    <TransitionHeight $height={`${height}px`}>
      <div ref={ref}>{footer}</div>
    </TransitionHeight>
  );
}
