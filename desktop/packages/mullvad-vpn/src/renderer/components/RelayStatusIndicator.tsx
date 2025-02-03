import styled from 'styled-components';
import { Styles } from 'styled-components/dist/types';

import { Colors } from '../lib/foundations';
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
  backgroundColor: props.$active ? Colors.green90 : Colors.red95,
}));

const TickIcon = styled(Cell.Icon)({
  marginLeft: '3px',
  marginRight: '8px',
});

interface IProps {
  active: boolean;
  selected: boolean;
}

export default function RelayStatusIndicator(props: IProps) {
  return props.selected ? (
    <TickIcon tintColor={Colors.white} source="icon-tick" width={18} />
  ) : (
    <StyledRelayStatus $active={props.active} />
  );
}

export const SpecialLocationIndicator = styled.div(indicatorStyles, {
  backgroundColor: Colors.white90,
});
