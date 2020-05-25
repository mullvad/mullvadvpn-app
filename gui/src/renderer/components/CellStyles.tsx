import React from 'react';
import styled from 'styled-components';
import { colors } from '../../config.json';
import ImageView, { IImageViewProps } from './ImageView';

export const StyledContainer = styled.div({
  display: 'flex',
  backgroundColor: colors.blue,
  alignItems: 'center',
  paddingLeft: '16px',
  paddingRight: '12px',
});

export const StyledSection = styled.div({
  display: 'flex',
  flexDirection: 'column',
});

export const StyledSectionTitle = styled.span({
  backgroundColor: colors.blue,
  padding: '14px 24px',
  marginBottom: '1px',
  fontFamily: 'DINPro',
  fontSize: '20px',
  fontWeight: 900,
  lineHeight: '26px',
  color: colors.white,
});

interface IStyledCellButtonProps {
  selected?: boolean;
  containedInSection: boolean;
}

export const StyledCellButton = styled.button({}, (props: IStyledCellButtonProps) => ({
  display: 'flex',
  padding: '0 16px',
  marginBottom: '1px',
  flex: 1,
  alignItems: 'center',
  alignContent: 'center',
  cursor: 'default',
  border: 'none',
  backgroundColor: props.selected
    ? colors.green
    : props.containedInSection
    ? colors.blue40
    : colors.blue,
  ':not(:disabled):hover': {
    backgroundColor: props.selected ? colors.green : colors.blue80,
  },
}));

export const StyledLabel = styled.div({}, (props: { disabled: boolean }) => ({
  margin: '14px 0 14px 8px',
  flex: 1,
  fontFamily: 'DINPro',
  fontSize: '20px',
  fontWeight: 900,
  lineHeight: '26px',
  letterSpacing: -0.2,
  color: props.disabled ? colors.white40 : colors.white,
  textAlign: 'left',
}));

export const StyledSubText = styled.span({
  color: colors.white60,
  fontFamily: 'Open Sans',
  fontSize: '13px',
  fontWeight: 800,
  flex: -1,
  textAlign: 'right',
  marginLeft: '8px',
});

export const StyledIcon = styled(ImageView)({
  marginLeft: '8px',
});

export const StyledTintedIcon = styled(StyledIcon).attrs((props: IImageViewProps) => ({
  tintColor: props.tintColor ?? colors.white60,
  tintHoverColor: props.tintHoverColor ?? props.tintColor ?? colors.white60,
}))((props: IImageViewProps) => ({
  [StyledCellButton + ':hover &']: {
    backgroundColor: props.tintHoverColor,
  },
}));

export const StyledFooter = styled.div({
  padding: '8px 24px 24px',
});

export const StyledFooterText = styled.span({
  fontFamily: 'Open Sans',
  fontSize: '13px',
  fontWeight: 600,
  lineHeight: '20px',
  letterSpacing: -0.2,
  color: colors.white80,
});

export const StyledFooterBoldText = styled(StyledFooterText)({
  fontWeight: 900,
});

export const StyledInputFrame = styled.div({
  flexGrow: 0,
  backgroundColor: 'rgba(255,255,255,0.1)',
  borderRadius: '4px',
  padding: '4px',
});

const inputTextStyles: React.CSSProperties = {
  fontFamily: 'Open Sans',
  fontSize: '20px',
  fontWeight: 600,
  lineHeight: '26px',
  height: '26px',
  textAlign: 'right',
  padding: '0px',
};

export const StyledInput = styled.input({}, (props: { valid?: boolean }) => ({
  ...inputTextStyles,
  backgroundColor: 'transparent',
  border: 'none',
  width: '100%',
  height: '100%',
  color: props.valid !== false ? colors.white : colors.red,
  '::placeholder': {
    color: colors.white60,
  },
}));

export const StyledAutoSizingTextInputContainer = styled.div({
  position: 'relative',
});

export const StyledAutoSizingTextInputFiller = styled.pre({
  ...inputTextStyles,
  minWidth: '80px',
  color: 'transparent',
});

export const StyledAutoSizingTextInputWrapper = styled.div({
  position: 'absolute',
  top: '0px',
  left: '0px',
  width: '100%',
  height: '100%',
});
