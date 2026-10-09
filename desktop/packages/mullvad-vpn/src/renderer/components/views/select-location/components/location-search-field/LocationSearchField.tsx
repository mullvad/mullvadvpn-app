import React from 'react';

import { messages } from '../../../../../../shared/gettext';
import { SearchTextField } from '../../../../search-text-field';
import { useScrollPositionContext } from '../../ScrollPositionContext';
import { useSelectLocationViewContext } from '../../SelectLocationViewContext';

export function LocationSearchField() {
  const { setSearchTerm } = useSelectLocationViewContext();
  const { resetScrollPositions } = useScrollPositionContext();
  const [searchValue, setSearchValue] = React.useState('');

  const handleInputValueChange = React.useCallback((value: string) => {
    setSearchValue(value);
  }, []);

  const deferredSearchValue = React.useDeferredValue(searchValue);

  React.useEffect(() => {
    if (deferredSearchValue.length < 2) {
      setSearchTerm('');
    } else {
      resetScrollPositions();
      setSearchTerm(deferredSearchValue.toLowerCase());
    }
  }, [deferredSearchValue, resetScrollPositions, setSearchTerm]);

  return (
    <SearchTextField variant="secondary" value={searchValue} onValueChange={handleInputValueChange}>
      <SearchTextField.InputGroup>
        <SearchTextField.InputGroup.Icon icon="search" />
        <SearchTextField.InputGroup.Input
          autoFocus
          placeholder={
            // TRANSLATORS: Placeholder text for search field in select location view
            messages.gettext('Search locations or servers')
          }
        />
        <SearchTextField.InputGroup.ClearButton />
      </SearchTextField.InputGroup>
    </SearchTextField>
  );
}
