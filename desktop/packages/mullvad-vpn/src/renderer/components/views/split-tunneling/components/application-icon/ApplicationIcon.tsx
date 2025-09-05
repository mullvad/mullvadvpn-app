import styled from 'styled-components';

import { type IApplication } from '../../../../../../shared/application-types';
import { spacings } from '../../../../../lib/foundations';
import { CellImage } from '../../../../cell';
import { disabledApplication, type DisabledApplicationProps } from '../../utils';

export const StyledIcon = styled(CellImage)<DisabledApplicationProps>(disabledApplication, {
  marginRight: spacings.small,
});

export const StyledIconPlaceholder = styled.div({
  width: '35px',
  marginRight: spacings.small,
});

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
