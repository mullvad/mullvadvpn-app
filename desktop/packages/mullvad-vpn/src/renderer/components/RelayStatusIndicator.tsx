import styled from 'styled-components';
import { Styles } from 'styled-components/dist/types';

import { colors, spacings } from '../lib/foundations';
import * as Cell from './cell';

const indicatorStyles: Styles<
  React.DetailedHTMLProps<React.HTMLAttributes<HTMLDivElement>, HTMLDivElement>
> = {
  width: '16px',
  height: '16px',
  borderRadius: '8px',
  margin: '0 12px 0 4px',
};

const StyledRelayStatus = styled.div<{ $active: boolean }>(indicatorStyles, (props) => ({
  backgroundColor: props.$active ? colors.green : colors.red,
}));

const TickIcon = styled(Cell.CellIcon)({
  marginLeft: '3px',
  marginRight: spacings.small,
});

interface IProps {
  active: boolean;
  selected: boolean;
}

export default function RelayStatusIndicator(props: IProps) {
  return props.selected ? (
    <TickIcon color="white" icon="checkmark" />
  ) : (
    <StyledRelayStatus $active={props.active} />
  );
}

export const SpecialLocationIndicator = styled.div(indicatorStyles, {
  backgroundColor: colors.white,
});
