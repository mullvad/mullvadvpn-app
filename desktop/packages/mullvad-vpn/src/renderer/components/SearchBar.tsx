import { useCallback, useEffect } from 'react';
import styled from 'styled-components';

import { messages } from '../../shared/gettext';
import { Icon, IconButton } from '../lib/components';
import { colors } from '../lib/foundations';
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
  color: colors.whiteAlpha60,
  backgroundColor: colors.whiteOnDarkBlue10,
  '&&::placeholder': {
    color: colors.whiteOnDarkBlue60,
  },
  '&&:focus': {
    color: colors.blue,
    backgroundColor: colors.white,
  },
});

export const StyledClearButton = styled(IconButton)({
  position: 'absolute',
  top: '50%',
  transform: 'translateY(-50%)',
  right: '9px',
});

export const StyledClearIcon = styled(Icon)({
  background: colors.whiteOnDarkBlue60,
  '&&:hover': {
    backgroundColor: colors.whiteOnDarkBlue40,
  },
});

export const StyledSearchIcon = styled(Icon)({
  position: 'absolute',
  top: '50%',
  transform: 'translateY(-50%)',
  left: '8px',
  [`${StyledSearchInput}:focus ~ &&`]: {
    backgroundColor: colors.blue,
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
      <StyledSearchIcon icon="search" color="whiteAlpha60" />
      {props.searchTerm.length > 0 && (
        <StyledClearButton variant="secondary" onClick={onClear}>
          <StyledClearIcon icon="cross-circle" />
        </StyledClearButton>
      )}
    </StyledSearchContainer>
  );
}
