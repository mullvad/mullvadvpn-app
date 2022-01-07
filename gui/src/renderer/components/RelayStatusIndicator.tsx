import styled from 'styled-components';
import { colors } from '../../config.json';
import * as Cell from './cell';

const StyledRelayStatus = styled.div((props: { active: boolean }) => ({
  width: '16px',
  height: '16px',
  borderRadius: '8px',
  margin: '0 12px 0 4px',
  backgroundColor: props.active ? colors.green90 : colors.red95,
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
    <TickIcon tintColor={colors.white} source="icon-tick" width={18} />
  ) : (
    <StyledRelayStatus active={props.active} />
  );
}
