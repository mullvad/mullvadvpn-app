import React from 'react';
import { sprintf } from 'sprintf-js';

import { messages } from '../../../../../../shared/gettext';
import { useCustomLists } from '../../../../../features/location/hooks';
import { Dialog, type DialogProps } from '../../../../../lib/components/dialog';
import { FlexColumn } from '../../../../../lib/components/flex-column';
import { formatHtml } from '../../../../../lib/html-formatter';
import type { GeographicalLocation } from '../../select-location-types';
import { SelectList } from '../select-list';

type AddToListDialogProps = Omit<DialogProps, 'children'> & {
  location: GeographicalLocation;
};

export function AddToListDialog({ location, onOpenChange, open }: AddToListDialogProps) {
  const { customLists } = useCustomLists();

  const handleClickCancel = React.useCallback(() => {
    onOpenChange?.(false);
  }, [onOpenChange]);

  let locationType: string;
  if (location.type === 'relay') {
    locationType =
      // TRANSLATORS: This refers to our VPN relays/servers
      messages.pgettext('select-location-view', 'Relay');
  } else if (location.type === 'city') {
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
                  messages.pgettext('select-location-view', 'Add %(locationType)s to list'),
                  {
                    locationType,
                  },
                ),
              )}
            </Dialog.Text>
            <FlexColumn gap="small">
              {customLists.map((list) => (
                <SelectList key={list.id} list={list} location={location} />
              ))}
            </FlexColumn>
            <Dialog.Button key="cancel" onClick={handleClickCancel}>
              <Dialog.Button.Text>{messages.gettext('Cancel')}</Dialog.Button.Text>
            </Dialog.Button>
          </Dialog.PopupContent>
        </Dialog.Popup>
      </Dialog.Portal>
    </Dialog>
  );
}
