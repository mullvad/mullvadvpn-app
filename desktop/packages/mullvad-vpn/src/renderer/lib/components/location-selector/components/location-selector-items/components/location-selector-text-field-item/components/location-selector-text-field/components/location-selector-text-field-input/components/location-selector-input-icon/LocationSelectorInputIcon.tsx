import type { TextFieldIconProps } from '../../../../../../../../../../../text-field/components';
import { LocationSelectorIcon } from '../../../../../../../../../locations-selector-icon';
import { useIsLocationSelected } from '../../../../../../../../hooks';
import { useLocationSelectorTextFieldItemContext } from '../../../../../../LocationSelectorTextFieldItemContext';
import { useGetLocationIcon, useGetLocationIconColor } from './hooks';

export type LocationSelectorInputIconProps = Omit<TextFieldIconProps, 'icon'>;

export function LocationSelectorInputIcon(props: LocationSelectorInputIconProps) {
  const { type, id } = useLocationSelectorTextFieldItemContext();
  const selected = useIsLocationSelected(id);
  const iconColor = useGetLocationIconColor(selected);
  const icon = useGetLocationIcon(type);

  return <LocationSelectorIcon icon={icon} color={iconColor} horizontalOffset={-1} {...props} />;
}
