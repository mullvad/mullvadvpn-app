import styled from 'styled-components';

import { Colors } from '../../../foundations';
import { BodySmallSemiBold, BodySmallSemiBoldProps } from '../../typography';
import { useButtonContext } from '../ButtonContext';

export type ButtonTextProps = Omit<BodySmallSemiBoldProps, 'color'>;
export const StyledText = styled(BodySmallSemiBold)``;

export const ButtonText = (props: ButtonTextProps) => {
  const { disabled } = useButtonContext();
  return <StyledText color={disabled ? Colors.white40 : Colors.white} {...props} />;
};
