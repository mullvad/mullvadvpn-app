import React from 'react';

import { messages } from '../../../../../../shared/gettext';
import { useSelectedLocations } from '../../../../../features/locations/hooks';
import { Dialog } from '../../../../../lib/components/dialog';
import { Location } from '../location-list-item';
import { useHandleSelectEntryLocation } from '../location-lists/hooks';

export function AutomaticLocation() {
  const handleSelectEntryLocation = useHandleSelectEntryLocation();
  const { entry } = useSelectedLocations();

  const handleClickAutomatic = React.useCallback(async () => {
    await handleSelectEntryLocation('any');
  }, [handleSelectEntryLocation]);

  const [open, setOpen] = React.useState(false);

  const openDialog = React.useCallback(() => setOpen(true), []);

  const handleOpenChange = React.useCallback((isOpen: boolean) => {
    setOpen(isOpen);
  }, []);

  return (
    <>
      <Location selected={entry === 'any'}>
        <Location.ListItem>
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
              onClick={openDialog}
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
      <Dialog open={open} onOpenChange={handleOpenChange}>
        <Dialog.Portal>
          <Dialog.Popup>
            <Dialog.PopupContent>
              <Dialog.Icon icon="info-circle" />
              <Dialog.TextGroup>
                <Dialog.Text>
                  {
                    // TRANSLATORS: Text in dialog explaning the behavior of the “Automatic” location option
                    messages.pgettext(
                      'select-location-view',
                      'When the “Automatic” location is selected, the app automatically picks a random entry server, prioritizing those closer to the exit location for better performance.',
                    )
                  }
                </Dialog.Text>
                <Dialog.Text>
                  {
                    // TRANSLATORS: Text in dialog warning that any enabled filters are ignored for the entry server when the “Automatic” location is selected
                    messages.pgettext(
                      'select-location-view',
                      'Attention: With the “Automatic” location, any enabled filters are ignored for the entry server.',
                    )
                  }
                </Dialog.Text>
              </Dialog.TextGroup>
              <Dialog.CloseButton>
                <Dialog.CloseButton.Text>{messages.gettext('Got it!')}</Dialog.CloseButton.Text>
              </Dialog.CloseButton>
            </Dialog.PopupContent>
          </Dialog.Popup>
        </Dialog.Portal>
      </Dialog>
    </>
  );
}
