import styled from 'styled-components';

import { spacings } from '../../../../../../../../../../foundations';
import { TextField } from '../../../../../../../../../text-field';
import type { TextFieldSupportingTextProps } from '../../../../../../../../../text-field/components';

export type LocationSelectorTextFieldSupportingTextProps = TextFieldSupportingTextProps;

export const StyledLocationSelectorTextFieldSupportingText = styled(TextField.SupportingText)`
  margin-left: ${spacings.big};
`;

export function LocationSelectorTextFieldSupportingText(
  props: LocationSelectorTextFieldSupportingTextProps,
) {
  return <StyledLocationSelectorTextFieldSupportingText {...props} />;
}
