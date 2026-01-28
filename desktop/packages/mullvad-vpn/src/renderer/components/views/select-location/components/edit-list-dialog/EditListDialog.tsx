import { useCallback, useState } from 'react';
import styled from 'styled-components';

import { ICustomList } from '../../../../../../shared/daemon-rpc-types';
import { messages } from '../../../../../../shared/gettext';
import log from '../../../../../../shared/logging';
import { useAppContext } from '../../../../../context';
import { Button } from '../../../../../lib/components';
import { colors } from '../../../../../lib/foundations';
import { useBoolean } from '../../../../../lib/utility-hooks';
import { tinyText } from '../../../../common-styles';
import { ModalAlert, ModalMessage } from '../../../../Modal';
import SimpleInput from '../../../../SimpleInput';

const StyledInputErrorText = styled.span(tinyText, {
  marginTop: '6px',
  color: colors.red,
});

const StyledModalMessage = styled(ModalMessage)({
  marginTop: '8px',
  marginBottom: '8px',
});

interface EditListProps {
  list: ICustomList;
  isOpen: boolean;
  hide: () => void;
}

// Dialog for changing the name of a custom list.
export function EditListDialog(props: EditListProps) {
  const { hide } = props;

  const { updateCustomList } = useAppContext();

  const [newName, setNewName] = useState(props.list.name);
  const newNameTrimmed = newName.trim();
  const newNameValid = newNameTrimmed !== '';
  const [error, setError, unsetError] = useBoolean();

  // Update name in list and save it.
  const save = useCallback(async () => {
    if (newNameValid) {
      try {
        const updatedList = { ...props.list, name: newNameTrimmed };
        const result = await updateCustomList(updatedList);
        if (result && result.type === 'name already exists') {
          setError();
        } else {
          hide();
        }
      } catch (e) {
        const error = e as Error;
        log.error(`Failed to edit custom list ${props.list.id}: ${error.message}`);
      }
    }
  }, [newNameValid, props.list, newNameTrimmed, updateCustomList, setError, hide]);

  // Errors should be reset when editing the value
  const onChange = useCallback(
    (value: string) => {
      setNewName(value);
      unsetError();
    },
    [unsetError],
  );

  return (
    <ModalAlert
      isOpen={props.isOpen}
      buttons={[
        <Button key="save" disabled={!newNameValid} onClick={save}>
          <Button.Text>{messages.gettext('Save')}</Button.Text>
        </Button>,
        <Button key="cancel" onClick={props.hide}>
          <Button.Text>{messages.gettext('Cancel')}</Button.Text>
        </Button>,
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
