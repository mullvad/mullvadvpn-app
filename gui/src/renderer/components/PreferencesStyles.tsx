import styled from 'styled-components';

import { colors } from '../../config.json';
import { InfoIcon } from './InfoButton';
import { Container } from './Layout';

export const StyledContainer = styled(Container)({
  backgroundColor: colors.darkBlue,
});

export const StyledContent = styled.div({
  display: 'flex',
  flexDirection: 'column',
  flex: 1,
  marginBottom: '2px',
});

export const StyledSeparator = styled.div((props: { height?: number }) => ({
  height: `${props.height ?? 1}px`,
}));

export const StyledInfoIcon = styled(InfoIcon)({
  marginRight: '16px',
});
