import React, { useContext } from 'react';
import styled from 'styled-components';

import { Icon, IconProps, Image, ImageProps } from '../../lib/components';
import { Colors, Spacings } from '../../lib/foundations';
import { buttonText, normalText, tinyText } from '../common-styles';
import { CellButton } from './CellButton';
import { CellDisabledContext } from './Container';

const StyledLabel = styled.div<{ disabled: boolean }>(buttonText, (props) => ({
  display: 'flex',
  margin: '10px 0',
  flex: 1,
  color: props.disabled ? Colors.white40 : Colors.white,
  textAlign: 'left',

  [`${LabelContainer} &&`]: {
    marginTop: '0px',
    marginBottom: 0,
    height: '20px',
    lineHeight: '20px',
  },

  [`${LabelContainer}:has(${StyledSubLabel}) &&`]: {
    marginTop: '5px',
  },
}));

const StyledSubText = styled.span<{ disabled: boolean }>(tinyText, (props) => ({
  color: props.disabled ? Colors.white20 : Colors.white60,
  flex: -1,
  textAlign: 'right',
  margin: `0 ${Spacings.spacing3}`,
}));

const StyledImage = styled(Image)<ImageProps & { disabled?: boolean }>(({ disabled }) => ({
  opacity: disabled ? 0.4 : 1,
}));

const StyledIcon = styled(Icon)<IconProps & { disabled?: boolean }>(({ disabled }) => ({
  opacity: disabled ? 0.4 : 1,
}));

const StyledTintedIcon = styled(Icon)<IconProps & { disabled?: boolean }>(
  ({ color, disabled }) => ({
    opacity: disabled ? 0.4 : 1,
    '&&:hover': {
      backgroundColor: color,
    },
    [`${CellButton}:not(:disabled):hover &&`]: {
      backgroundColor: color,
    },
  }),
);

const StyledSubLabel = styled.div<{ disabled: boolean }>(tinyText, {
  display: 'flex',
  alignItems: 'center',
  color: Colors.white60,
  marginBottom: '5px',
  lineHeight: '14px',
  height: '14px',
});

export const LabelContainer = styled.div({
  display: 'flex',
  flexDirection: 'column',
  flex: 1,
  minWidth: 0,
});

export function Label(props: React.HTMLAttributes<HTMLDivElement>) {
  const disabled = useContext(CellDisabledContext);
  return <StyledLabel disabled={disabled} {...props} />;
}

export function InputLabel(props: React.LabelHTMLAttributes<HTMLLabelElement>) {
  const disabled = useContext(CellDisabledContext);
  return <StyledLabel as="label" disabled={disabled} {...props} />;
}

export const ValueLabel = styled(Label)(normalText, {
  fontWeight: 400,
});

export function SubText(props: React.HTMLAttributes<HTMLDivElement>) {
  const disabled = useContext(CellDisabledContext);
  return <StyledSubText disabled={disabled} {...props} />;
}

export function CellImage(props: ImageProps) {
  const disabled = useContext(CellDisabledContext);
  return <StyledImage disabled={disabled} {...props} />;
}

export function CellIcon(props: IconProps) {
  const disabled = useContext(CellDisabledContext);
  return <StyledIcon disabled={disabled} {...props} />;
}

export function CellTintedIcon(props: IconProps) {
  const disabled = useContext(CellDisabledContext);
  return <StyledTintedIcon disabled={disabled} {...props} />;
}

export function SubLabel(props: React.HTMLAttributes<HTMLDivElement>) {
  const disabled = useContext(CellDisabledContext);
  return <StyledSubLabel disabled={disabled} {...props} />;
}
