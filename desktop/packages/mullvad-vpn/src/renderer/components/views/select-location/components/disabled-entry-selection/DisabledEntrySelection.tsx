import { useCallback } from 'react';
import { sprintf } from 'sprintf-js';

import { strings } from '../../../../../../shared/constants';
import { messages } from '../../../../../../shared/gettext';
import { RoutePath } from '../../../../../../shared/routes';
import { Button } from '../../../../../lib/components';
import { FlexColumn } from '../../../../../lib/components/flex-column';
import { useHistory } from '../../../../../lib/history';
import { StyledSelectionUnavailableText } from '../../SelectLocationStyles';

export function DisabledEntrySelection() {
  const { push } = useHistory();

  const multihop = messages.pgettext('settings-view', 'Multihop');
  const directOnly = messages.gettext('Direct only');

  const navigateToDaitaSettings = useCallback(() => {
    push(RoutePath.daitaSettings);
  }, [push]);

  return (
    <FlexColumn gap="large" margin={{ horizontal: 'large', bottom: 'tiny' }}>
      <StyledSelectionUnavailableText>
        {sprintf(
          messages.pgettext(
            'select-location-view',
            'The entry server for %(multihop)s is currently overridden by %(daita)s. To select an entry server, please first enable “%(directOnly)s” or disable "%(daita)s" in the settings.',
          ),
          { daita: strings.daita, multihop, directOnly },
        )}
      </StyledSelectionUnavailableText>
      <Button onClick={navigateToDaitaSettings}>
        <Button.Text>
          {sprintf(messages.pgettext('select-location-view', 'Open %(daita)s settings'), {
            daita: strings.daita,
          })}
        </Button.Text>
      </Button>
    </FlexColumn>
  );
}
