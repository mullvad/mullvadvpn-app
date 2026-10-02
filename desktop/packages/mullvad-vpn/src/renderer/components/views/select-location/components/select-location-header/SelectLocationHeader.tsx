import styled from 'styled-components';

import { messages } from '../../../../../../shared/gettext';
import { FlexColumn } from '../../../../../lib/components/flex-column';
import { colors, spacings } from '../../../../../lib/foundations';
import { AppNavigationHeader } from '../../../../app-navigation-header';
import { HeaderMenuIconButton } from '../header-menu-icon-button';

export const StyledSelectLocationHeader = styled.div`
  pointer-events: auto;
  background-color: ${colors.darkBlue};
  padding-bottom: ${spacings.small};
`;

export function SelectLocationHeader({ children }: React.PropsWithChildren) {
  return (
    <StyledSelectLocationHeader>
      <AppNavigationHeader
        title={
          // TRANSLATORS: Title label in navigation bar
          messages.pgettext('select-location-nav', 'Select location')
        }
        titleVisible>
        <HeaderMenuIconButton />
      </AppNavigationHeader>
      <FlexColumn margin={{ horizontal: 'medium' }}>{children}</FlexColumn>
    </StyledSelectLocationHeader>
  );
}
