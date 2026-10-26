import { Icon, type IconProps } from '../../../icon';
import { useFeatureIndicatorContext } from '../../FeatureIndicatorContext';

export type FeatureIndicatorIconProps = IconProps;

export function FeatureIndicatorIcon(props: FeatureIndicatorIconProps) {
  const { disabled } = useFeatureIndicatorContext();

  return <Icon color={disabled ? 'whiteAlpha40' : 'white'} size="tiny" {...props} />;
}
