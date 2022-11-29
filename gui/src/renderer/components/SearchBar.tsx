import { useCallback, useEffect, useRef } from 'react';
import styled from 'styled-components';

import { colors } from '../../config.json';
import { messages } from '../../shared/gettext';
import { normalText } from './common-styles';
import ImageView from './ImageView';

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
  color: colors.white60,
  backgroundColor: colors.white10,
  '::placeholder': {
    color: colors.white60,
  },
  ':focus': {
    color: colors.blue,
    backgroundColor: colors.white,
    '::placeholder': {
      color: colors.blue40,
    },
  },
});

export const StyledClearButton = styled.button({
  position: 'absolute',
  top: '50%',
  transform: 'translateY(-50%)',
  right: '9px',
  border: 'none',
  background: 'none',
  padding: 0,
});

export const StyledSearchIcon = styled(ImageView)({
  position: 'absolute',
  top: '50%',
  transform: 'translateY(-50%)',
  left: '9px',
  [`${StyledSearchInput}:focus ~ &`]: {
    backgroundColor: colors.blue,
  },
});

export const StyledClearIcon = styled(ImageView)({
  ':hover': {
    backgroundColor: colors.white60,
  },
  [`${StyledSearchInput}:focus ~ ${StyledClearButton} &`]: {
    backgroundColor: colors.blue40,
    ':hover': {
      backgroundColor: colors.blue,
    },
  },
});

interface ISearchBarProps {
  searchTerm: string;
  onSearch: (searchTerm: string) => void;
  className?: string;
  disableAutoFocus?: boolean;
}

export default function SearchBar(props: ISearchBarProps) {
  const inputRef = useRef() as React.RefObject<HTMLInputElement>;

  const onInput = useCallback(
    (event: React.FormEvent) => {
      const element = event.target as HTMLInputElement;
      props.onSearch(element.value);
    },
    [props.onSearch],
  );

  const onClear = useCallback(() => {
    props.onSearch('');
    inputRef.current?.blur();
  }, [props.onSearch]);

  useEffect(() => {
    if (!props.disableAutoFocus) {
      inputRef.current?.focus({ preventScroll: true });
    }
  }, []);

  return (
    <StyledSearchContainer className={props.className}>
      <StyledSearchInput
        ref={inputRef}
        value={props.searchTerm}
        onInput={onInput}
        placeholder={messages.gettext('Search for...')}
      />
      <StyledSearchIcon source="icon-search" width={24} tintColor={colors.white60} />
      {props.searchTerm.length > 0 && (
        <StyledClearButton onClick={onClear}>
          <StyledClearIcon source="icon-close" width={18} tintColor={colors.white40} />
        </StyledClearButton>
      )}
    </StyledSearchContainer>
  );
}
