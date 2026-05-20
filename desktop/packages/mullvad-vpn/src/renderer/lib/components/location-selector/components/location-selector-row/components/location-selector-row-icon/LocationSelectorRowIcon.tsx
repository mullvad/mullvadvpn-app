import {
  LocationSelectorIcon,
  type LocationSelectorIconProps,
} from '../../../locations-selector-icon';
import { useLocationSelectorRowContext } from '../../LocationSelectorRowContext';

export type LocationSelectorRowIconProps = LocationSelectorIconProps;

export function LocationSelectorRowIcon(props: LocationSelectorRowIconProps) {
  const { position } = useLocationSelectorRowContext();

  return (
    <LocationSelectorIcon
      color="whiteAlpha60"
      backgroundColor="darkBlue"
      position={position}
      {...props}
    />
  );
}
