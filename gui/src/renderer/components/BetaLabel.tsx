import styled from 'styled-components';
import { colors } from '../../config.json';
import { messages } from '../../shared/gettext';

const StyledBetaLabel = styled.span({
  display: 'inline-block',
  fontFamily: 'Open Sans',
  color: colors.blue,
  fontSize: '13px',
  fontWeight: 800,
  lineHeight: '20px',
  padding: '2px 0',
  background: colors.yellow,
  borderRadius: '5px',
  width: '50px',
  textAlign: 'center',
});

interface IBetaLabelProps {
  className?: string;
}

export default function BetaLabel(props: IBetaLabelProps) {
  return <StyledBetaLabel {...props}>{messages.gettext('BETA')}</StyledBetaLabel>;
}
