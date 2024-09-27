import React, { useEffect } from 'react';
import styled from 'styled-components';

import { colors } from '../../config.json';
import { useEffectEvent, useStyledRef } from '../lib/utility-hooks';
import * as AppButton from './AppButton';
import { measurements, normalText, tinyText } from './common-styles';
import CustomScrollbars from './CustomScrollbars';
import ImageView from './ImageView';
import { BackAction } from './KeyboardNavigation';
import { SmallButtonGrid } from './SmallButton';

export enum ModalAlertType {
  info = 1,
  caution,
  warning,

  loading,
  success,
  failure,
}

const modalClosedStyle = {
  transform: 'translateY(-45%) scale(80%)',
  opacity: 0,
};

const backdropClosedStyle = {
  backgroundColor: 'rgba(0,0,0,0)',
  backdropFilter: 'blur(0)',
};

const transitionStyle = { transition: 'all 150ms ease-out allow-discrete' };

const StyledModalAlert = styled.dialog(modalClosedStyle, transitionStyle, {
  zIndex: 100,
  top: '50%',
  width: '94vw',
  maxHeight: '80vh',
  margin: '0 auto',
  flexDirection: 'column',
  border: 'none',
  borderRadius: '11px',
  padding: '16px 0 16px 16px',
  backgroundColor: colors.darkBlue,
  boxShadow: ' 0px 15px 35px 5px rgba(0,0,0,0.5)',

  '&&[open]': {
    display: 'flex',
    transform: 'translateY(calc(-50% - 10px)) scale(100%)',
    opacity: 1,
    '@starting-style': modalClosedStyle,
  },

  '&&::backdrop': { ...backdropClosedStyle, ...transitionStyle },
  '&&[open]::backdrop': {
    backgroundColor: 'rgba(0,0,0,0.5)',
    backdropFilter: 'blur(1.5px)',

    '@starting-style': backdropClosedStyle,
  },
});

const StyledCustomScrollbars = styled(CustomScrollbars)({
  paddingRight: '16px',
});

const StyledModalAlertIcon = styled.div({
  display: 'flex',
  justifyContent: 'center',
  marginTop: '8px',
});

const ModalAlertButtonGroupContainer = styled.div({
  marginTop: measurements.buttonVerticalMargin,
});

const StyledSmallButtonGrid = styled(SmallButtonGrid)({
  marginRight: '16px',
});

const ModalAlertButtonContainer = styled.div({
  display: 'flex',
  flexDirection: 'column',
  marginRight: '16px',
});

interface ModalAlertProps {
  isOpen: boolean;
  type?: ModalAlertType;
  iconColor?: string;
  title?: string;
  message?: string | Array<string>;
  buttons?: React.ReactNode[];
  gridButtons?: React.ReactNode[];
  children?: React.ReactNode;
  close: () => void;
}

export function ModalAlert(props: ModalAlertProps) {
  const dialogRef = useStyledRef<HTMLDialogElement>();

  const messages = typeof props.message === 'string' ? [props.message] : props.message;

  const toggleModal = useEffectEvent((isOpen: boolean) => {
    if (isOpen) {
      dialogRef.current?.showModal();
    } else {
      dialogRef.current?.close();
    }
  });

  useEffect(() => {
    toggleModal(props.isOpen);
  }, [props.isOpen]);

  useEffect(() => () => dialogRef.current?.close());

  return (
    <BackAction action={props.close} disabled={!props.isOpen}>
      <StyledModalAlert ref={dialogRef}>
        <StyledCustomScrollbars>
          {props.type && (
            <StyledModalAlertIcon>
              <ModalAlertIcon type={props.type} iconColor={props.iconColor} />
            </StyledModalAlertIcon>
          )}
          {props.title && <ModalTitle>{props.title}</ModalTitle>}
          {messages &&
            messages.map((message) => <ModalMessage key={message}>{message}</ModalMessage>)}
          {props.children}
        </StyledCustomScrollbars>

        <ModalAlertButtonGroupContainer>
          {props.gridButtons && <StyledSmallButtonGrid>{props.gridButtons}</StyledSmallButtonGrid>}
          {props.buttons && (
            <AppButton.ButtonGroup>
              {props.buttons.map((button, index) => (
                <ModalAlertButtonContainer key={index}>{button}</ModalAlertButtonContainer>
              ))}
            </AppButton.ButtonGroup>
          )}
        </ModalAlertButtonGroupContainer>
      </StyledModalAlert>
    </BackAction>
  );
}

interface ModalElertIconProps {
  type: ModalAlertType;
  iconColor?: string;
}

function ModalAlertIcon(props: ModalElertIconProps) {
  let source = '';
  let color = undefined;
  switch (props.type) {
    case ModalAlertType.info:
      source = 'icon-info';
      color = colors.white;
      break;
    case ModalAlertType.caution:
      source = 'icon-alert';
      color = colors.white;
      break;
    case ModalAlertType.warning:
      source = 'icon-alert';
      color = colors.red;
      break;

    case ModalAlertType.loading:
      source = 'icon-spinner';
      break;
    case ModalAlertType.success:
      source = 'icon-success';
      break;
    case ModalAlertType.failure:
      source = 'icon-fail';
      break;
  }

  return <ImageView height={44} width={44} source={source} tintColor={props.iconColor ?? color} />;
}

const ModalTitle = styled.h1(normalText, {
  color: colors.white,
  fontWeight: 600,
  margin: '18px 0 0 0',
});

export const ModalMessage = styled.span(tinyText, {
  color: colors.white80,
  marginTop: '16px',

  [`${ModalTitle} ~ &&`]: {
    marginTop: '6px',
  },
});

export const ModalMessageList = styled.ul({
  listStyle: 'disc outside',
  paddingLeft: '20px',
  color: colors.white80,
});
