import React from 'react';

import { messages } from '../../../../../../shared/gettext';
import { ListItem } from '../../../../../lib/components/list-item';
import { AddCustomListFormProvider, useAddCustomListFormContext } from './AddCustomListFormContext';
import {
  useHandleClickCreateCustomList,
  useHandleCustomListNameChange,
  useHandleSubmitAddCustomList,
} from './hooks';

function AddListFormImpl() {
  const itemRef = React.useRef<HTMLDivElement>(null);
  const descriptionId = React.useId();

  const {
    formRef,
    inputRef,
    form: {
      error,
      customListTextField: { value, invalid, dirty, invalidReason },
    },
  } = useAddCustomListFormContext();

  const handleOnValueChange = useHandleCustomListNameChange();
  const handleSubmit = useHandleSubmitAddCustomList();
  const handleClick = useHandleClickCreateCustomList();

  return (
    <ListItem ref={itemRef}>
      <ListItem.Item>
        <ListItem.TextField
          formRef={formRef}
          value={value}
          onValueChange={handleOnValueChange}
          onSubmit={handleSubmit}
          invalid={dirty && (error || invalid)}>
          <ListItem.TextField.Input
            ref={inputRef}
            width="medium"
            maxLength={30}
            autoFocus
            aria-label={messages.pgettext('accessibility', 'Custom list name')}
            aria-describedby={descriptionId}
            aria-errormessage={invalidReason ? descriptionId : undefined}
          />
        </ListItem.TextField>
      </ListItem.Item>
      <ListItem.Trigger onClick={handleClick} disabled={invalid}>
        <ListItem.TrailingAction
          aria-label={messages.pgettext('accessibility', 'Create custom list')}>
          <ListItem.TrailingAction.Icon icon="checkmark" />
        </ListItem.TrailingAction>
      </ListItem.Trigger>
      <ListItem.Footer>
        <ListItem.FooterText id={descriptionId} role="status">
          {invalidReason
            ? invalidReason
            : messages.pgettext('select-location-view', 'Enter a name for the custom list')}
        </ListItem.FooterText>
      </ListItem.Footer>
    </ListItem>
  );
}

export function AddListForm() {
  return (
    <AddCustomListFormProvider>
      <AddListFormImpl />
    </AddCustomListFormProvider>
  );
}
