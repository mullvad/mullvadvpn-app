import React from 'react';
import styled from 'styled-components';

import { DeprecatedColors } from '../../../foundations';
import { BodySmallSemiBold, BodySmallSemiBoldProps } from '../../typography';
import { useButtonContext } from '../ButtonContext';

export type ButtonTextProps<T extends React.ElementType = 'span'> = BodySmallSemiBoldProps<T>;
export const StyledButtonText = styled(BodySmallSemiBold)``;

export function ButtonText<T extends React.ElementType = 'span'>(props: ButtonTextProps<T>) {
  const { disabled } = useButtonContext();
  return (
    <StyledButtonText
      color={disabled ? DeprecatedColors.white40 : DeprecatedColors.white}
      {...props}
    />
  );
}
