import styled from 'styled-components';

import { Spinner, type SpinnerProps } from '../../../spinner';

export type AlertSpinnerProps = SpinnerProps;

export const StyledAlertSpinner = styled(Spinner)`
  align-self: center;
`;

export function AlertSpinner(props: AlertSpinnerProps) {
  return <StyledAlertSpinner size="big" aria-hidden="true" {...props} />;
}
