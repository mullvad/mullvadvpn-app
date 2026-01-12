import styled from 'styled-components';

import { spacings } from '../../../../foundations';
import { IconButton, IconButtonProps } from '../../../icon-button';

export type TextFieldIconButtonProps = IconButtonProps;

export const StyledTextFieldIconButton = styled(IconButton)`
  position: absolute;
  top: 50%;
  transform: translateY(-50%);
  right: ${spacings.small};
`;

const TextFieldIconButton = (props: TextFieldIconButtonProps) => {
  return <StyledTextFieldIconButton size="small" {...props} />;
};

const TextFieldIconButtonNamespace = Object.assign(TextFieldIconButton, {
  Icon: IconButton.Icon,
});
export { TextFieldIconButtonNamespace as TextFieldIconButton };
