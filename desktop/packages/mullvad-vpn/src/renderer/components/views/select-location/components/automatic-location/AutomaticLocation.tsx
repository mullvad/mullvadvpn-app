import React from 'react';

import { messages } from '../../../../../../shared/gettext';
import { useEntryLocation } from '../../../../../features/locations/hooks';
import type { ListItemProps } from '../../../../../lib/components/list-item';
import { Location } from '../location-list-item';
import { useHandleSelectEntryLocation } from '../location-lists/hooks';
import { AutomaticEntryInfoDialog } from './components';

export type AutomaticLocationProps = {
  position?: ListItemProps['position'];
};

export function AutomaticLocation({ position }: AutomaticLocationProps) {
  const handleSelectEntryLocation = useHandleSelectEntryLocation();
  const { entryLocation } = useEntryLocation();

  const [infoDialogOpen, setInfoDialogOpen] = React.useState(false);
  const openInfoDialog = React.useCallback(() => setInfoDialogOpen(true), []);

  const handleClickAutomatic = React.useCallback(async () => {
    await handleSelectEntryLocation('any');
  }, [handleSelectEntryLocation]);

  return (
    <>
      <Location selected={entryLocation === 'any'}>
        <Location.ListItem position={position}>
          <Location.ListItem.Trigger
            onClick={handleClickAutomatic}
            aria-label={messages.pgettext('accessibility', 'Use automatic entry location')}>
            <Location.ListItem.Item>
              <Location.ListItem.Item.Label>
                {messages.gettext('Automatic')}
              </Location.ListItem.Item.Label>
            </Location.ListItem.Item>
          </Location.ListItem.Trigger>
          <Location.ListItem.TrailingActions>
            <Location.ListItem.Trigger
              onClick={openInfoDialog}
              aria-label={messages.pgettext(
                'accessibility',
                'Read more about the automatic entry location',
              )}>
              <Location.ListItem.TrailingActions.Action>
                <Location.ListItem.TrailingActions.Action.Icon icon="info-circle" />
              </Location.ListItem.TrailingActions.Action>
            </Location.ListItem.Trigger>
          </Location.ListItem.TrailingActions>
        </Location.ListItem>
      </Location>
      <AutomaticEntryInfoDialog open={infoDialogOpen} onOpenChange={setInfoDialogOpen} />
    </>
  );
}
