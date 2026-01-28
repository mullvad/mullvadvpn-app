import { useCallback, useEffect, useState } from 'react';
import styled from 'styled-components';

import { CustomListError } from '../../../../../../shared/daemon-rpc-types';
import { messages } from '../../../../../../shared/gettext';
import log from '../../../../../../shared/logging';
import { colors } from '../../../../../lib/foundations';
import { useBoolean, useStyledRef } from '../../../../../lib/utility-hooks';
import Accordion from '../../../../Accordion';
import * as Cell from '../../../../cell';
import { measurements } from '../../../../common-styles';
import { BackAction } from '../../../../keyboard-navigation';
import SimpleInput from '../../../../SimpleInput';

const StyledCellContainer = styled(Cell.Container)({
  padding: 0,
  background: 'none',
});

const StyledInputContainer = styled.div({
  display: 'flex',
  alignItems: 'center',
  flex: 1,
  backgroundColor: colors.blue,
  paddingLeft: measurements.horizontalViewMargin,
  height: measurements.rowMinHeight,
});

const StyledCellButton = styled(Cell.SideButton)({
  border: 'none',
});

const StyledAddListCellButton = styled(StyledCellButton)({
  marginLeft: 'auto',
});

const StyledSideButtonIcon = styled(Cell.CellIcon)({
  [`${StyledCellButton}:disabled &&, ${StyledAddListCellButton}:disabled &&`]: {
    backgroundColor: colors.whiteAlpha40,
  },

  [`${StyledCellButton}:not(:disabled):hover &&, ${StyledAddListCellButton}:not(:disabled):hover &&`]:
    {
      backgroundColor: colors.white,
    },
});

const StyledInput = styled(SimpleInput)<{ $error: boolean }>((props) => ({
  color: props.$error ? colors.red : 'auto',
}));

interface AddListFormProps {
  visible: boolean;
  onCreateList: (list: string) => Promise<void | CustomListError>;
  cancel: () => void;
}

export function AddListForm(props: AddListFormProps) {
  const { onCreateList, cancel } = props;

  const [name, setName] = useState('');
  const nameTrimmed = name.trim();
  const nameValid = nameTrimmed !== '';
  const [error, setError, unsetError] = useBoolean();
  const containerRef = useStyledRef<HTMLDivElement>();
  const inputRef = useStyledRef<HTMLInputElement>();

  // Errors should be reset when editing the value
  const onChange = useCallback(
    (value: string) => {
      setName(value);
      unsetError();
    },
    [unsetError],
  );

  const createList = useCallback(async () => {
    if (nameValid) {
      try {
        const result = await onCreateList(nameTrimmed);
        if (result) {
          setError();
        }
      } catch (e) {
        const error = e as Error;
        log.error('Failed to create list:', error.message);
      }
    }
  }, [nameValid, onCreateList, nameTrimmed, setError]);

  const onBlur = useCallback(
    (event: React.FocusEvent<HTMLInputElement>) => {
      // Only cancel if losing focus to something else than the contents of the row container.
      if (!event.relatedTarget || !containerRef.current?.contains(event.relatedTarget)) {
        cancel();
      }
    },
    [containerRef, cancel],
  );

  const onTransitionEnd = useCallback(() => {
    if (!props.visible) {
      setName('');
    }
  }, [props.visible]);

  useEffect(() => {
    if (props.visible) {
      inputRef.current?.focus();
    }
  }, [inputRef, props.visible]);

  return (
    <BackAction disabled={!props.visible} action={props.cancel}>
      <Accordion expanded={props.visible} onTransitionEnd={onTransitionEnd}>
        <StyledCellContainer ref={containerRef}>
          <StyledInputContainer>
            <StyledInput
              ref={inputRef}
              value={name}
              onChangeValue={onChange}
              onSubmitValue={createList}
              onBlur={onBlur}
              maxLength={30}
              $error={error}
              autoFocus
            />
          </StyledInputContainer>

          <StyledAddListCellButton
            $backgroundColor={colors.blue}
            $backgroundColorHover={colors.blue80}
            disabled={!nameValid}
            onClick={createList}>
            <StyledSideButtonIcon icon="checkmark" color="whiteAlpha60" />
          </StyledAddListCellButton>
        </StyledCellContainer>
        <Cell.CellFooter>
          <Cell.CellFooterText>
            {messages.pgettext('select-location-view', 'List names must be unique.')}
          </Cell.CellFooterText>
        </Cell.CellFooter>
      </Accordion>
    </BackAction>
  );
}
