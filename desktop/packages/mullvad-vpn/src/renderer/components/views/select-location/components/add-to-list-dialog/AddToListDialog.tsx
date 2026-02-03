import React, { useCallback } from 'react';
import { sprintf } from 'sprintf-js';

import { ICustomList, RelayLocationGeographical } from '../../../../../../shared/daemon-rpc-types';
import { messages } from '../../../../../../shared/gettext';
import log from '../../../../../../shared/logging';
import { useAppContext } from '../../../../../context';
import { useCustomLists } from '../../../../../features/location/hooks';
import { Dialog, type DialogProps } from '../../../../../lib/components/dialog';
import { FlexColumn } from '../../../../../lib/components/flex-column';
import { formatHtml } from '../../../../../lib/html-formatter';
import { SelectList } from '../select-list';

type AddToListDialogProps = Omit<DialogProps, 'children'> & {
  location: RelayLocationGeographical;
};

export function AddToListDialog({ location, onOpenChange, open }: AddToListDialogProps) {
  const { updateCustomList } = useAppContext();
  const { customLists } = useCustomLists();

  const close = React.useCallback(() => {
    onOpenChange?.(false);
  }, [onOpenChange]);

  const addLocationToCustomList = useCallback(
    async (list: ICustomList) => {
      const updatedList = {
        ...list,
        locations: [...list.locations, location],
      };
      try {
        await updateCustomList(updatedList);
      } catch (e) {
        const error = e as Error;
        log.error(`Failed to edit custom list ${list.id}: ${error.message}`);
      }

      close();
    },
    [close, location, updateCustomList],
  );

  let locationType: string;
  if ('hostname' in location) {
    locationType =
      // TRANSLATORS: This refers to our VPN relays/servers
      messages.pgettext('select-location-view', 'Relay');
  } else if ('city' in location) {
    locationType = messages.pgettext('select-location-view', 'City');
  } else {
    locationType = messages.pgettext('select-location-view', 'Country');
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <Dialog.Portal>
        <Dialog.Popup>
          <Dialog.PopupContent>
            <Dialog.Text>
              {formatHtml(
                sprintf(
                  // TRANSLATORS: This is a label shown above a list of options.
                  // TRANSLATORS: Available placeholder:
                  // TRANSLATORS: %(locationType) - Could be either "Country", "City" and "Relay"
                  messages.pgettext('select-location-view', 'Add <b>%(locationType)s</b> to list'),
                  {
                    locationType,
                  },
                ),
              )}
            </Dialog.Text>
            <FlexColumn gap="small">
              {customLists.map((list) => (
                <SelectList
                  key={list.id}
                  list={list}
                  location={location}
                  add={addLocationToCustomList}
                />
              ))}
            </FlexColumn>
            <Dialog.Button key="cancel" onClick={close}>
              <Dialog.Button.Text>{messages.gettext('Cancel')}</Dialog.Button.Text>
            </Dialog.Button>
          </Dialog.PopupContent>
        </Dialog.Popup>
      </Dialog.Portal>
    </Dialog>
  );
}
