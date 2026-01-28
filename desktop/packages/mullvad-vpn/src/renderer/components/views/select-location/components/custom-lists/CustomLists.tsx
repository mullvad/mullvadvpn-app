import { useCallback } from 'react';
import styled from 'styled-components';

import { CustomListError, type RelayLocation } from '../../../../../../shared/daemon-rpc-types';
import { messages } from '../../../../../../shared/gettext';
import { useAppContext } from '../../../../../context';
import { colors } from '../../../../../lib/foundations';
import { useBoolean } from '../../../../../lib/utility-hooks';
import Accordion from '../../../../Accordion';
import * as Cell from '../../../../cell';
import { measurements } from '../../../../common-styles';
import { useRelayListContext } from '../../RelayListContext';
import { useScrollPositionContext } from '../../ScrollPositionContext';
import { useSelectLocationContext } from '../../SelectLocationView';
import { AddListForm } from '../add-list-form/AddListForm';
import { RelayLocationList } from '../relay-location-list';

const StyledCellContainer = styled(Cell.Container)({
  padding: 0,
  background: 'none',
});

const StyledHeaderLabel = styled(Cell.Label)({
  display: 'block',
  flex: 1,
  backgroundColor: colors.blue,
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

const StyledSideButtonIcon = styled(Cell.CellIcon)({
  [`${StyledCellButton}:disabled &&, ${StyledAddListCellButton}:disabled &&`]: {
    backgroundColor: colors.whiteAlpha40,
  },

  [`${StyledCellButton}:not(:disabled):hover &&, ${StyledAddListCellButton}:not(:disabled):hover &&`]:
    {
      backgroundColor: colors.white,
    },
});

interface CustomListsProps {
  selectedElementRef: React.Ref<HTMLDivElement>;
  onSelect: (value: RelayLocation) => void;
}

export function CustomLists(props: CustomListsProps) {
  const [addListVisible, showAddList, hideAddList] = useBoolean();
  const { createCustomList } = useAppContext();
  const { searchTerm } = useSelectLocationContext();
  const { customLists } = useRelayListContext();

  const createList = useCallback(
    async (name: string): Promise<void | CustomListError> => {
      const result = await createCustomList({
        name,
        locations: [],
      });
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
          $backgroundColor={colors.blue}
          $backgroundColorHover={colors.blue80}
          onClick={showAddList}>
          <StyledSideButtonIcon icon="add-circle" color="whiteAlpha60" />
        </StyledCellButton>
      </StyledCellContainer>

      <Accordion expanded>
        <CustomListsImpl selectedElementRef={props.selectedElementRef} onSelect={props.onSelect} />
      </Accordion>

      <AddListForm visible={addListVisible} onCreateList={createList} cancel={hideAddList} />
    </Cell.Group>
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
