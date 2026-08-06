import styled from 'styled-components';

import { Spinner, type SpinnerProps } from '../../../spinner';

export type ButtonSpinnerProps = SpinnerProps;

export const StyledButtonSpinner = styled(Spinner)``;

export function ButtonSpinner(props: ButtonSpinnerProps) {
  return <StyledButtonSpinner size="medium" aria-hidden="true" {...props} />;
}
