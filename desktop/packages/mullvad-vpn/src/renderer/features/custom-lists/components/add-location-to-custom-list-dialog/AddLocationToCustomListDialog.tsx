import React from 'react';
import { sprintf } from 'sprintf-js';

import { messages } from '../../../../../shared/gettext';
import { Dialog, type DialogProps } from '../../../../lib/components/dialog';
import { FlexColumn } from '../../../../lib/components/flex-column';
import { formatHtml } from '../../../../lib/html-formatter';
import type { GeographicalLocation } from '../../../locations/types';
import { useCustomLists } from '../../hooks';
import { AddLocationToCustomListListItem } from './components';
import { useLocationTypeMessage } from './hooks';

type AddToCustomListDialog = Omit<DialogProps, 'children'> & {
  location: GeographicalLocation;
};

export function AddLocationToCustomListDialog({
  location,
  onOpenChange,
  open,
}: AddToCustomListDialog) {
  const { customLists } = useCustomLists();

  const handleClickCancel = React.useCallback(() => {
    onOpenChange?.(false);
  }, [onOpenChange]);

  const locationTypeMessage = useLocationTypeMessage(location);

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
                  messages.pgettext('custom-list-feature', 'Add %(locationType)s to list'),
                  {
                    locationType: locationTypeMessage,
                  },
                ),
              )}
            </Dialog.Text>
            <FlexColumn gap="small">
              {customLists.map((list) => (
                <AddLocationToCustomListListItem key={list.id} list={list} location={location} />
              ))}
            </FlexColumn>
            <Dialog.Button onClick={handleClickCancel}>
              <Dialog.Button.Text>{messages.gettext('Close')}</Dialog.Button.Text>
            </Dialog.Button>
          </Dialog.PopupContent>
        </Dialog.Popup>
      </Dialog.Portal>
    </Dialog>
  );
}
