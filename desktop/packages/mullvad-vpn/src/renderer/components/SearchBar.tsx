import { useCallback, useEffect } from 'react';
import styled from 'styled-components';

import { messages } from '../../shared/gettext';
import { Colors } from '../lib/foundations';
import { useEffectEvent, useStyledRef } from '../lib/utility-hooks';
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
  lineHeight: '24px',
  color: Colors.white60,
  backgroundColor: Colors.white10,
  '&&::placeholder': {
    color: Colors.white60,
  },
  '&&:focus': {
    color: Colors.blue,
    backgroundColor: Colors.white,
  },
  '&&:focus::placeholder': {
    color: Colors.blue40,
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
  [`${StyledSearchInput}:focus ~ &&`]: {
    backgroundColor: Colors.blue,
  },
});

export const StyledClearIcon = styled(ImageView)({
  '&&:hover': {
    backgroundColor: Colors.white60,
  },
  [`${StyledSearchInput}:focus ~ ${StyledClearButton} &&`]: {
    backgroundColor: Colors.blue40,
  },
  [`${StyledSearchInput}:focus ~ ${StyledClearButton} &&:hover`]: {
    backgroundColor: Colors.blue,
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
      <StyledSearchIcon source="icon-search" width={24} tintColor={Colors.white60} />
      {props.searchTerm.length > 0 && (
        <StyledClearButton onClick={onClear}>
          <StyledClearIcon source="icon-close" width={18} tintColor={Colors.white40} />
        </StyledClearButton>
      )}
    </StyledSearchContainer>
  );
}
