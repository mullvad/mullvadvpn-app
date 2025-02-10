import { useCallback, useEffect, useState } from 'react';
import styled from 'styled-components';

import {
  CustomListError,
  type CustomLists,
  type RelayLocation,
} from '../../../shared/daemon-rpc-types';
import { messages } from '../../../shared/gettext';
import log from '../../../shared/logging';
import { useAppContext } from '../../context';
import { Colors } from '../../lib/foundations';
import { useBoolean, useStyledRef } from '../../lib/utility-hooks';
import Accordion from '../Accordion';
import * as Cell from '../cell';
import { measurements } from '../common-styles';
import { BackAction } from '../KeyboardNavigation';
import SimpleInput from '../SimpleInput';
import { useRelayListContext } from './RelayListContext';
import RelayLocationList from './RelayLocationList';
import { useScrollPositionContext } from './ScrollPositionContext';
import { useSelectLocationContext } from './SelectLocationContainer';

const StyledCellContainer = styled(Cell.Container)({
  padding: 0,
  background: 'none',
});

const StyledInputContainer = styled.div({
  display: 'flex',
  alignItems: 'center',
  flex: 1,
  backgroundColor: Colors.blue,
  paddingLeft: measurements.horizontalViewMargin,
  height: measurements.rowMinHeight,
});

const StyledHeaderLabel = styled(Cell.Label)({
  display: 'block',
  flex: 1,
  backgroundColor: Colors.blue,
  paddingLeft: measurements.horizontalViewMargin,
  margin: 0,
  height: measurements.rowMinHeight,
  lineHeight: measurements.rowMinHeight,
});

const StyledCellButton = styled(Cell.SideButton)({
  border: 'none',
});

const StyledAddListCellButton = styled(StyledCellButton)({
  marginLeft: 'auto',
});

const StyledSideButtonIcon = styled(Cell.Icon)({
  padding: '3px',

  [`${StyledCellButton}:disabled &&, ${StyledAddListCellButton}:disabled &&`]: {
    backgroundColor: Colors.white40,
  },

  [`${StyledCellButton}:not(:disabled):hover &&, ${StyledAddListCellButton}:not(:disabled):hover &&`]:
    {
      backgroundColor: Colors.white,
    },
});

const StyledInput = styled(SimpleInput)<{ $error: boolean }>((props) => ({
  color: props.$error ? Colors.red : 'auto',
}));

interface CustomListsProps {
  selectedElementRef: React.Ref<HTMLDivElement>;
  onSelect: (value: RelayLocation) => void;
}

export default function CustomLists(props: CustomListsProps) {
  const [addListVisible, showAddList, hideAddList] = useBoolean();
  const { createCustomList } = useAppContext();
  const { searchTerm } = useSelectLocationContext();
  const { customLists } = useRelayListContext();

  const createList = useCallback(
    async (name: string): Promise<void | CustomListError> => {
      const result = await createCustomList(name);
      // If an error is returned it should be passed as the return value.
      if (result) {
        return result;
      }

      hideAddList();
    },
    [createCustomList, hideAddList],
  );

  if (searchTerm !== '' && !customLists.some((list) => list.visible)) {
    return null;
  }

  return (
    <Cell.Group>
      <StyledCellContainer>
        <StyledHeaderLabel>
          {messages.pgettext('select-location-view', 'Custom lists')}
        </StyledHeaderLabel>
        <StyledCellButton
          $backgroundColor={Colors.blue}
          $backgroundColorHover={Colors.blue80}
          onClick={showAddList}>
          <StyledSideButtonIcon source="icon-add" tintColor={Colors.white60} width={18} />
        </StyledCellButton>
      </StyledCellContainer>

      <Accordion expanded>
        <CustomListsImpl selectedElementRef={props.selectedElementRef} onSelect={props.onSelect} />
      </Accordion>

      <AddListForm visible={addListVisible} onCreateList={createList} cancel={hideAddList} />
    </Cell.Group>
  );
}

interface AddListFormProps {
  visible: boolean;
  onCreateList: (list: string) => Promise<void | CustomListError>;
  cancel: () => void;
}

function AddListForm(props: AddListFormProps) {
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
            $backgroundColor={Colors.blue}
            $backgroundColorHover={Colors.blue80}
            disabled={!nameValid}
            onClick={createList}>
            <StyledSideButtonIcon source="icon-check" tintColor={Colors.white60} width={18} />
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

interface CustomListsImplProps {
  selectedElementRef: React.Ref<HTMLDivElement>;
  onSelect: (value: RelayLocation) => void;
}

function CustomListsImpl(props: CustomListsImplProps) {
  const { onSelect: propsOnSelect } = props;

  const { customLists, expandLocation, collapseLocation, onBeforeExpand } = useRelayListContext();
  const { resetHeight } = useScrollPositionContext();

  const onSelect = useCallback(
    (value: RelayLocation) => {
      const location = { ...value };
      if ('country' in location) {
        // Only the geographical part should be sent to the daemon when setting a location.
        delete location.customList;
      }
      propsOnSelect(location);
    },
    [propsOnSelect],
  );

  return (
    <RelayLocationList
      source={customLists}
      onExpand={expandLocation}
      onCollapse={collapseLocation}
      onWillExpand={onBeforeExpand}
      selectedElementRef={props.selectedElementRef}
      onSelect={onSelect}
      onTransitionEnd={resetHeight}
      allowAddToCustomList={false}
    />
  );
}
