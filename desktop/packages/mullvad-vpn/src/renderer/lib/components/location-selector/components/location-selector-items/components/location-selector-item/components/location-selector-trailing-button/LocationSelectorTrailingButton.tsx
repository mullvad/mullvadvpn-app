import styled from 'styled-components';

import { spacings } from '../../../../../../../../foundations';
import { IconButton, type IconButtonProps } from '../../../../../../../icon-button';

export type LocationSelectorTrailingButtonProps = IconButtonProps;

export const StyledLocationSelectorTrailingButton = styled(IconButton)`
  margin-right: ${spacings.tiny};
`;

function LocationSelectorTrailingButton(props: LocationSelectorTrailingButtonProps) {
  return <StyledLocationSelectorTrailingButton {...props} />;
}
const LocationSelectorTrailingButtonNamespace = Object.assign(LocationSelectorTrailingButton, {
  Icon: IconButton.Icon,
});

export { LocationSelectorTrailingButtonNamespace as LocationSelectorTrailingButton };
