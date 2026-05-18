import styled from 'styled-components';

import { Icon, IconProps } from '../../../icon';

export type TextFieldIconProps = IconProps;

export const StyledTextFieldIcon = styled(Icon)`
  position: absolute;
  top: 50%;
  transform: translateY(-50%);
  left: 7px;
`;

export const TextFieldIcon = (props: TextFieldIconProps) => {
  return <StyledTextFieldIcon size="small" {...props} />;
};
