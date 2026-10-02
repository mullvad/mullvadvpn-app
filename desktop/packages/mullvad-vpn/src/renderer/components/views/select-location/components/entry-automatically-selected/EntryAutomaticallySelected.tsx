import React from 'react';
import styled from 'styled-components';

import { messages } from '../../../../../../shared/gettext';
import { LocationType } from '../../../../../features/locations/types';
import { useMultihop } from '../../../../../features/multihop/hooks';
import { BodySmall, Button, Icon } from '../../../../../lib/components';
import { FlexColumn } from '../../../../../lib/components/flex-column';
import { useSelectLocationViewContext } from '../../SelectLocationViewContext';

const StyledGrid = styled.div`
  display: grid;
  grid-template-rows: minmax(0, 1fr) auto minmax(0, 2fr);
  justify-items: center;

  flex-grow: 1;
`;

const StyledMiddleContent = styled(FlexColumn)`
  grid-row: 2;
  align-self: center;
`;

const StyledBottomContent = styled.div`
  grid-row: 3;
  align-self: end;
  width: 100%;
`;

export function EntryAutomaticallySelected() {
  const { setMultihop } = useMultihop();
  const { setLocationType } = useSelectLocationViewContext();

  const handleClickSetMultihopToAlways = React.useCallback(async () => {
    setLocationType(LocationType.exit);
    await setMultihop({
      multihop: 'always',
    });
  }, [setLocationType, setMultihop]);

  return (
    <StyledGrid>
      <StyledMiddleContent gap="medium" alignItems="center">
        <Icon icon="magic-multihop" size="big" />
        <FlexColumn gap="small">
          <BodySmall color="whiteAlpha60" textAlign="center">
            {
              // TRANSLATORS: Text explaining that the entry server is automatically selected based on the chosen location.
              messages.pgettext(
                'select-location-view',
                'The entry server is currently selected to automatically work with your selected location.',
              )
            }
          </BodySmall>
          <BodySmall color="whiteAlpha60" textAlign="center">
            {
              // TRANSLATORS: Text explaning that if the user want to manually select a server, they need to switch multihop mode.
              messages.pgettext(
                'select-location-view',
                'To manually select an entry server please switch multihop mode to "Always".',
              )
            }
          </BodySmall>
        </FlexColumn>
      </StyledMiddleContent>
      <StyledBottomContent>
        <Button width="fill" onClick={handleClickSetMultihopToAlways}>
          <Button.Text>
            {
              // TRANSLATORS: Text for a button that sets multihop mode to "Always".
              messages.pgettext('select-location-view', 'Set multihop to "Always"')
            }
          </Button.Text>
        </Button>
      </StyledBottomContent>
    </StyledGrid>
  );
}
