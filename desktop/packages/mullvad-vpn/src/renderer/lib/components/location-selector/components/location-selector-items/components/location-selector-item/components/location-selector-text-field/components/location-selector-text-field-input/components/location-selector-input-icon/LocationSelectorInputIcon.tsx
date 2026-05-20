import type { TextFieldIconProps } from '../../../../../../../../../../../text-field/components';
import { LocationSelectorIcon } from '../../../../../../../../../locations-selector-icon';
import { useIsLocationSelected } from '../../../../../../hooks';
import { useLocationSelectorItemContext } from '../../../../../../LocationSelectorItemContext';
import { useGetLocationIcon, useGetLocationIconColor } from './hooks';

export type LocationSelectorInputIconProps = Omit<TextFieldIconProps, 'icon'>;

export function LocationSelectorInputIcon(props: LocationSelectorInputIconProps) {
  const { type, id } = useLocationSelectorItemContext();
  const selected = useIsLocationSelected(id);
  const iconColor = useGetLocationIconColor(selected);
  const icon = useGetLocationIcon(type);
  const backgroundColor = selected ? 'blue40' : 'darkerBlue10';

  return (
    <LocationSelectorIcon
      icon={icon}
      color={iconColor}
      backgroundColor={backgroundColor}
      horizontalOffset={1}
      {...props}
    />
  );
}
