import { Info } from '../../../../../info';
import type { InfoButtonProps } from '../../../../../info/components/info-button';

export const AppNavigationHeaderInfoButton = (props: InfoButtonProps) => {
  return <Info.Button variant="secondary" {...props} />;
};
