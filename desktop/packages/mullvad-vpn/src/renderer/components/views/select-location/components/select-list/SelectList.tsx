import { useCallback } from 'react';
import styled from 'styled-components';

import {
  compareRelayLocationGeographical,
  ICustomList,
  RelayLocation,
} from '../../../../../../shared/daemon-rpc-types';
import { messages } from '../../../../../../shared/gettext';
import { colors } from '../../../../../lib/foundations';
import * as Cell from '../../../../cell';
import { normalText } from '../../../../common-styles';

const StyledSelectListItemLabel = styled(Cell.Label)(normalText, {
  fontWeight: 'normal',
});

const StyledSelectListItemIcon = styled(Cell.CellTintedIcon)({
  [`${Cell.CellButton}:not(:disabled):hover &&`]: {
    backgroundColor: colors.whiteAlpha80,
  },
});

interface SelectListProps {
  list: ICustomList;
  location: RelayLocation;
  add: (list: ICustomList) => void;
}

export function SelectList(props: SelectListProps) {
  const { add } = props;

  const onAdd = useCallback(() => add(props.list), [add, props.list]);

  // List should be disabled if location already is in list.
  const disabled = props.list.locations.some((location) =>
    compareRelayLocationGeographical(location, props.location),
  );

  return (
    <Cell.CellButton onClick={onAdd} disabled={disabled}>
      <StyledSelectListItemLabel>
        {props.list.name} {disabled && messages.pgettext('select-location-view', '(Added)')}
      </StyledSelectListItemLabel>
      <StyledSelectListItemIcon icon="add-circle" />
    </Cell.CellButton>
  );
}
