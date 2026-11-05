import { Icon, type IconProps } from '../../../../../../../lib/components';
import { useLocationContext } from '../../LocationContext';

export type LocationIconProps = IconProps;

export function LocationIcon(props: LocationIconProps) {
  const { selected } = useLocationContext();
  return <Icon size="medium" color={selected ? 'green' : 'whiteAlpha60'} aria-hidden {...props} />;
}
