import type { IconProps } from '../../../../../../../../../icon';
import { LocationSelectorIcon } from '../../../../../../../locations-selector-icon';
import { useIsLocationSelected } from '../../../../../../hooks';
import { useLocationSelectorButtonItemContext } from '../../../../LocationSelectorItemContext';

export type LocationSelectorButtonIconProps = IconProps;

export function LocationSelectorButtonIcon(props: LocationSelectorButtonIconProps) {
  const { id } = useLocationSelectorButtonItemContext();
  const selected = useIsLocationSelected(id);
  const backgroundColor = selected ? 'blue40' : 'darkerBlue10';

  return (
    <LocationSelectorIcon
      color="white"
      backgroundColor={backgroundColor}
      horizontalOffset={-9}
      {...props}
    />
  );
}
