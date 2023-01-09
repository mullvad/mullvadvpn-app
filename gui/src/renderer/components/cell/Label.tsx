import React, { useContext } from 'react';
import styled from 'styled-components';

import { colors } from '../../../config.json';
import { buttonText, tinyText } from '../common-styles';
import ImageView, { IImageViewProps } from '../ImageView';
import { CellButton } from './CellButton';
import { CellDisabledContext } from './Container';

const StyledLabel = styled.div(buttonText, (props: { disabled: boolean }) => ({
  margin: '10px 0',
  flex: 1,
  color: props.disabled ? colors.white40 : colors.white,
  textAlign: 'left',
}));

const StyledSubText = styled.span(tinyText, (props: { disabled: boolean }) => ({
  color: props.disabled ? colors.white20 : colors.white60,
  flex: -1,
  textAlign: 'right',
  marginLeft: '8px',
  marginRight: '8px',
}));

const StyledIconContainer = styled.div((props: { disabled: boolean }) => ({
  opacity: props.disabled ? 0.4 : 1,
}));

const StyledTintedIcon = styled(ImageView).attrs((props: IImageViewProps) => ({
  tintColor: props.tintColor ?? colors.white60,
  tintHoverColor: props.tintHoverColor ?? props.tintColor ?? colors.white60,
}))((props: IImageViewProps) => ({
  ':hover': {
    backgroundColor: props.tintColor,
  },
  [`${CellButton}:not(:disabled):hover &&`]: {
    backgroundColor: props.tintHoverColor,
  },
}));

export function Label(props: React.HTMLAttributes<HTMLDivElement>) {
  const disabled = useContext(CellDisabledContext);
  return <StyledLabel disabled={disabled} {...props} />;
}

export function InputLabel(props: React.LabelHTMLAttributes<HTMLLabelElement>) {
  const disabled = useContext(CellDisabledContext);
  return <StyledLabel as="label" disabled={disabled} {...props} />;
}

export function SubText(props: React.HTMLAttributes<HTMLDivElement>) {
  const disabled = useContext(CellDisabledContext);
  return <StyledSubText disabled={disabled} {...props} />;
}

export function UntintedIcon(props: IImageViewProps) {
  const disabled = useContext(CellDisabledContext);
  return (
    <StyledIconContainer disabled={disabled}>
      <ImageView {...props} />
    </StyledIconContainer>
  );
}

export function Icon(props: IImageViewProps) {
  const disabled = useContext(CellDisabledContext);
  return (
    <StyledIconContainer disabled={disabled}>
      <StyledTintedIcon {...props} />
    </StyledIconContainer>
  );
}
