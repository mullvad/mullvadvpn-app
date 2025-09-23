import React from 'react';
import styled from 'styled-components';

import { BodySmallSemiBold, BodySmallSemiBoldProps } from '../../../text';
import { useButtonContext } from '../../ButtonContext';

export type ButtonTextProps<T extends React.ElementType = 'span'> = BodySmallSemiBoldProps<T>;
export const StyledButtonText = styled(BodySmallSemiBold)``;

export function ButtonText<T extends React.ElementType = 'span'>(props: ButtonTextProps<T>) {
  const { disabled } = useButtonContext();
  return <StyledButtonText color={disabled ? 'whiteAlpha40' : 'white'} {...props} />;
}
