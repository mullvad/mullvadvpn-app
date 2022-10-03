import styled from 'styled-components';

import { colors } from '../../config.json';
import * as Cell from './cell';
import ImageView from './ImageView';

export const StyledCustomDnsFooter = styled(Cell.CellFooter)({
  marginBottom: '2px',
});

export const StyledAddCustomDnsButton = styled(Cell.CellButton)({
  backgroundColor: colors.blue40,
});

export const StyledAddCustomDnsLabel = styled(Cell.Label)(
  {},
  (props: { paddingLeft?: number }) => ({
    fontFamily: 'Open Sans',
    fontWeight: 400,
    fontSize: '16px',
    paddingLeft: (props.paddingLeft ?? 32) + 'px',
    whiteSpace: 'pre-wrap',
    overflowWrap: 'break-word',
    width: '171px',
    marginRight: '25px',
  }),
);

export const StyledContainer = styled(Cell.Container)({
  display: 'flex',
  backgroundColor: colors.blue40,
});

export const StyledButton = styled.button({
  display: 'flex',
  alignItems: 'center',
  flex: 1,
  border: 'none',
  background: 'transparent',
  padding: 0,
  margin: 0,
});

export const StyledLabel = styled(Cell.Label)({
  fontFamily: 'Open Sans',
  fontWeight: 400,
  fontSize: '16px',
  paddingLeft: '32px',
  whiteSpace: 'pre-wrap',
  overflowWrap: 'break-word',
  width: '171px',
  marginRight: '25px',
});

export const StyledRemoveButton = styled.button({
  background: 'transparent',
  border: 'none',
  padding: 0,
});

export const StyledRemoveIcon = styled(ImageView)({
  [StyledRemoveButton + ':hover &']: {
    backgroundColor: colors.white80,
  },
});
