import { useCallback, useState } from 'react';
import { sprintf } from 'sprintf-js';
import styled from 'styled-components';

import { colors } from '../../../config.json';
import {
  compareRelayLocationGeographical,
  ICustomList,
  RelayLocation,
  RelayLocationGeographical,
} from '../../../shared/daemon-rpc-types';
import { messages } from '../../../shared/gettext';
import log from '../../../shared/logging';
import { useAppContext } from '../../context';
import { formatHtml } from '../../lib/html-formatter';
import { useBoolean } from '../../lib/utilityHooks';
import { useSelector } from '../../redux/store';
import * as AppButton from '../AppButton';
import * as Cell from '../cell';
import { normalText, tinyText } from '../common-styles';
import { ModalAlert, ModalMessage } from '../Modal';
import SimpleInput from '../SimpleInput';

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
  const { updateCustomList } = useAppContext();
  const customLists = useSelector((state) => state.settings.customLists);

  const add = useCallback(
    (list: ICustomList) => {
      // Update the list with the new location.
      const updatedList = {
        ...list,
        locations: [...list.locations, props.location],
      };
      void updateCustomList(updatedList);
      props.hide();
    },
    [location, updateCustomList],
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
        <AppButton.BlueButton key="cancel" onClick={props.hide}>
          {messages.gettext('Cancel')}
        </AppButton.BlueButton>,
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

const StyledSelectListItemLabel = styled(Cell.Label)(normalText, {
  fontWeight: 'normal',
});

const StyledSelectListItemIcon = styled(Cell.Icon)({
  [`${Cell.CellButton}:not(:disabled):hover &&`]: {
    backgroundColor: colors.white80,
  },
});

interface SelectListProps {
  list: ICustomList;
  location: RelayLocation;
  add: (list: ICustomList) => void;
}

function SelectList(props: SelectListProps) {
  const onAdd = useCallback(() => props.add(props.list), [props.list]);

  // List should be disabled if location already is in list.
  const disabled = props.list.locations.some((location) =>
    compareRelayLocationGeographical(location, props.location),
  );

  return (
    <Cell.CellButton onClick={onAdd} disabled={disabled}>
      <StyledSelectListItemLabel>{props.list.name}</StyledSelectListItemLabel>
      <StyledSelectListItemIcon source="icon-add" width={18} />
    </Cell.CellButton>
  );
}

const StyledInputErrorText = styled.span(tinyText, {
  marginTop: '6px',
  color: colors.red,
});

interface EditListProps {
  list: ICustomList;
  isOpen: boolean;
  hide: () => void;
}

// Dialog for changing the name of a custom list.
export function EditListDialog(props: EditListProps) {
  const { updateCustomList } = useAppContext();

  const [newName, setNewName] = useState(props.list.name);
  const [error, setError, unsetError] = useBoolean();

  // Update name in list and save it.
  const save = useCallback(async () => {
    try {
      const updatedList = { ...props.list, name: newName };
      const result = await updateCustomList(updatedList);
      if (result && result.type === 'name already exists') {
        setError();
      } else {
        props.hide();
      }
    } catch (e) {
      const error = e as Error;
      log.error('Failed to create list:', error.message);
    }
  }, [props.list, newName, props.hide]);

  // Errors should be reset when editing the value
  const onChange = useCallback((value: string) => {
    setNewName(value);
    unsetError();
  }, []);

  return (
    <ModalAlert
      isOpen={props.isOpen}
      buttons={[
        <AppButton.BlueButton key="save" onClick={save}>
          {messages.gettext('Save')}
        </AppButton.BlueButton>,
        <AppButton.BlueButton key="cancel" onClick={props.hide}>
          {messages.gettext('Cancel')}
        </AppButton.BlueButton>,
      ]}
      close={props.hide}>
      <StyledModalMessage>
        {messages.pgettext('select-location-view', 'Edit list name')}
      </StyledModalMessage>
      <SimpleInput
        value={newName}
        onChangeValue={onChange}
        onSubmitValue={save}
        maxLength={30}
        autoFocus
      />
      {error && (
        <StyledInputErrorText>
          {messages.pgettext('select-location-view', 'Name is already taken.')}
        </StyledInputErrorText>
      )}
    </ModalAlert>
  );
}
