import React from 'react';
import styled from 'styled-components';

import { Colors } from '../../../foundations';
import { BodySmallSemiBold, BodySmallSemiBoldProps } from '../../typography';
import { useButtonContext } from '../ButtonContext';

export type ButtonTextProps<T extends React.ElementType> = BodySmallSemiBoldProps<T>;
export const StyledText = styled(BodySmallSemiBold)``;

export const ButtonText = <T extends React.ElementType = 'span'>(props: ButtonTextProps<T>) => {
  const { disabled } = useButtonContext();
  return <StyledText color={disabled ? Colors.white40 : Colors.white} {...props} />;
};
