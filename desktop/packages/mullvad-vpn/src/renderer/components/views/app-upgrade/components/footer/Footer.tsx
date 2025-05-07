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

const footers = {
  download: <DownloadFooter />,
  error: <ErrorFooter />,
  initial: <InitialFooter />,
  launch: <LaunchFooter />,
  pause: <PauseFooter />,
  verify: <VerifyFooter />,
};

export function Footer() {
  const step = useStep();
  const footer = footers[step];

  const [ref, { height }] = useMeasure<HTMLDivElement>();

  return (
    <TransitionHeight $height={`${height}px`}>
      <div ref={ref}>{footer}</div>
    </TransitionHeight>
  );
}
