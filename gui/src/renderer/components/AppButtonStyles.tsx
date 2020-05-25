import styled from 'styled-components';
import { colors } from '../../config.json';

export const StyledLabelContainer = styled.div((props: { textAdjustment: number }) => ({
  display: 'flex',
  flex: 1,
  paddingRight: `${props.textAdjustment > 0 ? props.textAdjustment : 0}px`,
  paddingLeft: `${props.textAdjustment < 0 ? Math.abs(props.textAdjustment) : 0}px`,
}));

export const StyledLabel = styled.span({
  fontFamily: 'DINPro',
  fontSize: '20px',
  fontWeight: 900,
  lineHeight: '26px',
  flex: 1,
  color: colors.white,
  textAlign: 'center',
});

export const StyledButton = styled.button({
  display: 'flex',
  cursor: 'default',
  borderRadius: 4,
  border: 'none',
  padding: 0,
  ':disabled': {
    opacity: 0.5,
  },
});

export const StyledButtonContent = styled.div({
  display: 'flex',
  flex: 1,
  flexDirection: 'row',
  alignItems: 'center',
  justifyContent: 'center',
  padding: 9,
});
