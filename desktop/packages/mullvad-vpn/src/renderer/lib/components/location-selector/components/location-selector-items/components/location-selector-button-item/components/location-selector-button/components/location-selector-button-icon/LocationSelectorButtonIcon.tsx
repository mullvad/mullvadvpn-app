import type { IconProps } from '../../../../../../../../../icon';
import { LocationSelectorIcon } from '../../../../../../../locations-selector-icon';

export type LocationSelectorButtonIconProps = IconProps;

export function LocationSelectorButtonIcon(props: LocationSelectorButtonIconProps) {
  return <LocationSelectorIcon color="white" horizontalOffset={-9} {...props} />;
}
