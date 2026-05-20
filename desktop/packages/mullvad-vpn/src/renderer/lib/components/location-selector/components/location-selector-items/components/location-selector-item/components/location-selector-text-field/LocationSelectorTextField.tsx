import styled from 'styled-components';

import { spacings } from '../../../../../../../../foundations';
import { TextField, type TextFieldProps } from '../../../../../../../text-field';
import {
  LocationSelectorTextFieldInput,
  LocationSelectorTextFieldSupportingText,
  StyledTextFieldInput,
} from './components';

export type LocationSelectorTextFieldProps = TextFieldProps;

export const StyledLocationSelectorTextField = styled(TextField)`
  ${StyledTextFieldInput} {
    padding-left: calc(${spacings.small} + 18px + ${spacings.tiny});
  }
`;

function LocationSelectorTextField(props: LocationSelectorTextFieldProps) {
  return <StyledLocationSelectorTextField variant="secondary" {...props} />;
}

const LocationSelectorTextFieldNamespace = Object.assign(LocationSelectorTextField, {
  Input: LocationSelectorTextFieldInput,
  SupportingText: LocationSelectorTextFieldSupportingText,
});

export { LocationSelectorTextFieldNamespace as LocationSelectorTextField };
