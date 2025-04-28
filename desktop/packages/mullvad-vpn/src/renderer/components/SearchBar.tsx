import { useCallback, useEffect } from 'react';
import styled from 'styled-components';

import { messages } from '../../shared/gettext';
import { Icon, IconButton } from '../lib/components';
import { DeprecatedColors } from '../lib/foundations';
import { useEffectEvent, useStyledRef } from '../lib/utility-hooks';
import { normalText } from './common-styles';

export const StyledSearchContainer = styled.div({
  position: 'relative',
  display: 'flex',
});

export const StyledSearchInput = styled.input.attrs({ type: 'text' })({
  ...normalText,
  flex: 1,
  border: 'none',
  borderRadius: '4px',
  padding: '9px 38px',
  margin: 0,
  lineHeight: '24px',
  color: DeprecatedColors.white60,
  backgroundColor: DeprecatedColors.white10,
  '&&::placeholder': {
    color: DeprecatedColors.white60,
  },
  '&&:focus': {
    color: DeprecatedColors.blue,
    backgroundColor: DeprecatedColors.white,
  },
  '&&:focus::placeholder': {
    color: DeprecatedColors.blue40,
  },
});

// TODO: The focus styling can be removed once we implement the new colors from foundations
export const StyledClearButton = styled(IconButton)({
  position: 'absolute',
  top: '50%',
  transform: 'translateY(-50%)',
  right: '9px',
  [`${StyledSearchInput}:focus ~ && > div`]: {
    backgroundColor: DeprecatedColors.blue40,
  },
});

export const StyledSearchIcon = styled(Icon)({
  position: 'absolute',
  top: '50%',
  transform: 'translateY(-50%)',
  left: '8px',
  [`${StyledSearchInput}:focus ~ &&`]: {
    backgroundColor: DeprecatedColors.blue,
  },
});

interface ISearchBarProps {
  searchTerm: string;
  onSearch: (searchTerm: string) => void;
  className?: string;
  disableAutoFocus?: boolean;
}

export default function SearchBar(props: ISearchBarProps) {
  const { onSearch } = props;

  const inputRef = useStyledRef<HTMLInputElement>();

  const onInput = useCallback(
    (event: React.FormEvent) => {
      const element = event.target as HTMLInputElement;
      onSearch(element.value);
    },
    [onSearch],
  );

  const onClear = useCallback(() => {
    onSearch('');
    inputRef.current?.blur();
  }, [inputRef, onSearch]);

  const focusInput = useEffectEvent(() => {
    if (!props.disableAutoFocus) {
      inputRef.current?.focus({ preventScroll: true });
    }
  });

  useEffect(() => focusInput(), []);

  return (
    <StyledSearchContainer className={props.className}>
      <StyledSearchInput
        ref={inputRef}
        value={props.searchTerm}
        onInput={onInput}
        placeholder={messages.gettext('Search for...')}
      />
      <StyledSearchIcon icon="search" color={DeprecatedColors.white60} />
      {props.searchTerm.length > 0 && (
        <StyledClearButton variant="secondary" onClick={onClear}>
          <IconButton.Icon icon="cross-circle" />
        </StyledClearButton>
      )}
    </StyledSearchContainer>
  );
}
