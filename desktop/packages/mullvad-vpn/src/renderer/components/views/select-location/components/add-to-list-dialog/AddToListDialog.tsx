import { useCallback } from 'react';
import { sprintf } from 'sprintf-js';
import styled from 'styled-components';

import { ICustomList, RelayLocationGeographical } from '../../../../../../shared/daemon-rpc-types';
import { messages } from '../../../../../../shared/gettext';
import log from '../../../../../../shared/logging';
import { useAppContext } from '../../../../../context';
import { Button } from '../../../../../lib/components';
import { formatHtml } from '../../../../../lib/html-formatter';
import { useSelector } from '../../../../../redux/store';
import { ModalAlert, ModalMessage } from '../../../../Modal';
import { SelectList } from '../select-list';

const StyledModalMessage = styled(ModalMessage)({
  marginTop: '8px',
  marginBottom: '8px',
});

interface AddToListDialogProps {
  location: RelayLocationGeographical;
  isOpen: boolean;
  hide: () => void;
}

// Dialog that displays list of custom lists when adding location to custom list.
export function AddToListDialog(props: AddToListDialogProps) {
  const { hide } = props;

  const { updateCustomList } = useAppContext();
  const customLists = useSelector((state) => state.settings.customLists);

  const add = useCallback(
    async (list: ICustomList) => {
      // Update the list with the new location.
      const updatedList = {
        ...list,
        locations: [...list.locations, props.location],
      };
      try {
        await updateCustomList(updatedList);
      } catch (e) {
        const error = e as Error;
        log.error(`Failed to edit custom list ${list.id}: ${error.message}`);
      }

      hide();
    },
    [hide, props.location, updateCustomList],
  );

  let locationType: string;
  if ('hostname' in props.location) {
    // TRANSLATORS: This refers to our VPN relays/servers
    locationType = messages.pgettext('select-location-view', 'Relay');
  } else if ('city' in props.location) {
    locationType = messages.pgettext('select-location-view', 'City');
  } else {
    locationType = messages.pgettext('select-location-view', 'Country');
  }

  const lists = customLists.map((list) => (
    <SelectList key={list.id} list={list} location={props.location} add={add} />
  ));

  return (
    <ModalAlert
      isOpen={props.isOpen}
      buttons={[
        <Button key="cancel" onClick={props.hide}>
          <Button.Text>{messages.gettext('Cancel')}</Button.Text>
        </Button>,
      ]}
      close={props.hide}>
      <StyledModalMessage>
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
      </StyledModalMessage>
      {lists}
    </ModalAlert>
  );
}
