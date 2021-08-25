import styled from 'styled-components';
import { colors } from '../../config.json';
import * as AppButton from './AppButton';
import * as Cell from './cell';
import { mediumText, smallText } from './common-styles';
import ImageView from './ImageView';
import { Container } from './Layout';
import { NavigationScrollbars } from './NavigationBar';

export const StyledPageCover = styled.div({}, (props: { show: boolean }) => ({
  position: 'absolute',
  zIndex: 2,
  top: 0,
  left: 0,
  right: 0,
  bottom: 0,
  backgroundColor: colors.black,
  opacity: 0.5,
  display: props.show ? 'block' : 'none',
}));

export const StyledContainer = styled(Container)({
  backgroundColor: colors.darkBlue,
});

export const StyledNavigationScrollbars = styled(NavigationScrollbars)({
  flex: 1,
});

export const StyledContent = styled.div({
  display: 'flex',
  flexDirection: 'column',
  flex: 1,
});

export const StyledCellButton = styled(Cell.CellButton)((props: { lookDisabled?: boolean }) => ({
  ':not(:disabled):hover': {
    backgroundColor: props.lookDisabled ? colors.blue : undefined,
  },
}));

const disabledApplication = (props: { lookDisabled?: boolean }) => ({
  opacity: props.lookDisabled ? 0.6 : undefined,
});

export const StyledIcon = styled(Cell.UntintedIcon)(disabledApplication, {
  marginRight: '12px',
});

export const StyledCellLabel = styled(Cell.Label)(disabledApplication, {
  fontFamily: 'Open Sans',
  fontWeight: 'normal',
  fontSize: '16px',
});

export const StyledIconPlaceholder = styled.div({
  width: '35px',
  marginRight: '12px',
});

export const StyledApplicationListContent = styled.div({
  display: 'flex',
  flexDirection: 'column',
});

export const StyledApplicationListAnimation = styled.div({}, (props: { height?: number }) => ({
  overflow: 'hidden',
  height: props.height ? `${props.height}px` : 'auto',
  transition: 'height 500ms ease-in-out',
  marginBottom: '20px',
}));

export const StyledSpinnerRow = styled.div({
  display: 'flex',
  justifyContent: 'center',
  padding: '8px 0',
  background: colors.blue40,
});

export const StyledBrowseButton = styled(AppButton.BlueButton)({
  margin: '0 22px 22px',
});

export const StyledCellContainer = styled(Cell.Container)({
  marginBottom: '20px',
});

export const StyledSearchContainer = styled.div({
  position: 'relative',
  marginBottom: '18px',
});

export const StyledSearchInput = styled.input.attrs({ type: 'text' })({
  ...mediumText,
  width: 'calc(100% - 22px * 2)',
  border: 'none',
  borderRadius: '4px',
  padding: '9px 38px',
  margin: '0 22px',
  color: colors.white60,
  backgroundColor: colors.white10,
  '::placeholder': {
    color: colors.white60,
  },
  ':focus': {
    color: colors.blue,
    backgroundColor: colors.white,
    '::placeholder': {
      color: colors.blue40,
    },
  },
});

export const StyledClearButton = styled.button({
  position: 'absolute',
  top: '50%',
  transform: 'translateY(-50%)',
  right: '28px',
  border: 'none',
  background: 'none',
  padding: 0,
});

export const StyledSearchIcon = styled(ImageView)({
  position: 'absolute',
  top: '50%',
  transform: 'translateY(-50%)',
  left: '28px',
  [`${StyledSearchInput}:focus ~ &`]: {
    backgroundColor: colors.blue,
  },
});

export const StyledClearIcon = styled(ImageView)({
  ':hover': {
    backgroundColor: colors.white60,
  },
  [`${StyledSearchInput}:focus ~ ${StyledClearButton} &`]: {
    backgroundColor: colors.blue40,
    ':hover': {
      backgroundColor: colors.blue,
    },
  },
});

export const StyledNoResult = styled(Cell.Footer)({
  display: 'flex',
  flexDirection: 'column',
  paddingTop: 0,
  marginTop: 0,
});

export const StyledNoResultText = styled(Cell.FooterText)({
  textAlign: 'center',
});

export const StyledNoResultSearchTerm = styled.span({
  fontWeight: 'bold',
});

export const StyledDisabledWarning = styled.span(smallText, {
  margin: '0 22px 18px',
  color: colors.red,
});
