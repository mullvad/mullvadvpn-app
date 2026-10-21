import styled from 'styled-components';

import { messages } from '../../../../../../../../shared/gettext';
import { LocationSelector } from '../../../../../../../lib/components/location-selector';
import type { LocationSelectorTextFieldItemProps } from '../../../../../../../lib/components/location-selector/components/location-selector-items/components';
import { TextField } from '../../../../../../../lib/components/text-field';
import { useSelectLocationViewContext } from '../../../../SelectLocationViewContext';
import {
  useHandleBlur,
  useHandleClearButtonClick,
  useHandleInputKeyDown,
  useHandleValueChange,
} from './hooks';
import { TextFieldItemProvider, useTextFieldItemContext } from './TextFieldItemContext';

export type SelectLocationSelectorItemProps = LocationSelectorTextFieldItemProps & {
  defaultValue?: string;
  placeholder?: string;
};

const StyledInput = styled(LocationSelector.Items.TextFieldItem.TextField.Input)`
  &&::-webkit-search-cancel-button {
    display: none;
  }
`;

function TextFieldItemImpl({
  id,
  placeholder,
  'aria-label': ariaLabel,
  ...props
}: Omit<SelectLocationSelectorItemProps, 'value' | 'inputRef' | 'delay'>) {
  const {
    triggerRef,
    textField: { inputRef, value, handleFocus },
  } = useTextFieldItemContext();
  const { isolatedItem } = useSelectLocationViewContext();

  const handleClearButtonClick = useHandleClearButtonClick();

  const handleKeyDown = useHandleInputKeyDown();
  const handleBlur = useHandleBlur();
  const handleValueChange = useHandleValueChange();

  const showClearButton = id === isolatedItem;

  return (
    <LocationSelector.Items.TextFieldItem
      id={id}
      inputRef={inputRef}
      triggerRef={triggerRef}
      {...props}>
      <LocationSelector.Items.TextFieldItem.Trigger aria-label={ariaLabel} onBlur={handleBlur}>
        <TextField.InputGroup>
          <LocationSelector.Items.TextFieldItem.TextField
            value={value}
            onValueChange={handleValueChange}>
            <StyledInput
              placeholder={placeholder}
              onFocus={handleFocus}
              onKeyDown={handleKeyDown}
              type="text"
            />
            {showClearButton && (
              <LocationSelector.Items.TextFieldItem.TextField.ClearButton
                onClick={handleClearButtonClick}
                aria-label={messages.pgettext('accessibility', 'Clear search')}
              />
            )}
          </LocationSelector.Items.TextFieldItem.TextField>
        </TextField.InputGroup>
      </LocationSelector.Items.TextFieldItem.Trigger>
    </LocationSelector.Items.TextFieldItem>
  );
}

export function TextFieldItem({ defaultValue, id, ...props }: SelectLocationSelectorItemProps) {
  return (
    <TextFieldItemProvider id={id} defaultValue={defaultValue}>
      <TextFieldItemImpl id={id} {...props} />
    </TextFieldItemProvider>
  );
}
