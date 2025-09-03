import { type IApplication } from '../../../../../../shared/application-types';
import { StyledIcon, StyledIconPlaceholder } from './ApplicationIconStyles';

export type ApplicationIconProps = {
  disabled?: boolean;
  icon?: IApplication['icon'];
};

export function ApplicationIcon({ disabled, icon }: ApplicationIconProps) {
  if (icon) {
    return <StyledIcon source={icon} width={35} height={35} $lookDisabled={disabled} />;
  }

  return <StyledIconPlaceholder />;
}
