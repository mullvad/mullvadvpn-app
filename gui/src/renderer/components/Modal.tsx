import React, { useCallback, useContext, useEffect, useMemo, useRef, useState } from 'react';
import ReactDOM from 'react-dom';
import styled from 'styled-components';

import { colors } from '../../config.json';
import log from '../../shared/logging';
import { useWillExit } from '../lib/will-exit';
import * as AppButton from './AppButton';
import { measurements, tinyText } from './common-styles';
import CustomScrollbars from './CustomScrollbars';
import ImageView from './ImageView';
import { BackAction } from './KeyboardNavigation';

const MODAL_CONTAINER_ID = 'modal-container';

const ModalContent = styled.div({
  position: 'absolute',
  display: 'flex',
  flexDirection: 'column',
  flex: 1,
  top: 0,
  left: 0,
  right: 0,
  bottom: 0,
  overflow: 'hidden',
});

const ModalBackground = styled.div({}, (props: { visible: boolean }) => ({
  backgroundColor: props.visible ? 'rgba(0,0,0,0.5)' : 'rgba(0,0,0,0)',
  backdropFilter: props.visible ? 'blur(1.5px)' : '',
  position: 'absolute',
  display: 'flex',
  flexDirection: 'column',
  flex: 1,
  top: 0,
  left: 0,
  right: 0,
  bottom: 0,
  transition: 'background-color 150ms ease-out',
  pointerEvents: props.visible ? 'auto' : 'none',
  zIndex: 2,
}));

export const StyledModalContainer = styled.div({
  position: 'relative',
  flex: 1,
});

interface IModalContainerProps {
  children?: React.ReactNode;
}

interface IModalContext {
  activeModal: boolean;
  setActiveModal: (value: boolean) => void;
  previousActiveElement: React.MutableRefObject<HTMLElement | undefined>;
}

const noActiveModalContextError = new Error('ActiveModalContext.Provider missing');
const ActiveModalContext = React.createContext<IModalContext>({
  get activeModal(): boolean {
    throw noActiveModalContextError;
  },
  setActiveModal(_value) {
    throw noActiveModalContextError;
  },
  get previousActiveElement(): React.MutableRefObject<HTMLElement | undefined> {
    throw noActiveModalContextError;
  },
});

export function ModalContainer(props: IModalContainerProps) {
  const [activeModal, setActiveModal] = useState(false);
  const previousActiveElement = useRef<HTMLElement>();

  const contextValue = useMemo(
    () => ({
      activeModal,
      setActiveModal,
      previousActiveElement,
    }),
    [activeModal],
  );

  useEffect(() => {
    if (!activeModal) {
      previousActiveElement.current?.focus();
    }
  }, [activeModal]);

  return (
    <ActiveModalContext.Provider value={contextValue}>
      <StyledModalContainer id={MODAL_CONTAINER_ID}>
        <ModalContent aria-hidden={activeModal}>{props.children}</ModalContent>
      </StyledModalContainer>
    </ActiveModalContext.Provider>
  );
}

export enum ModalAlertType {
  info = 1,
  caution,
  warning,
}

const ModalAlertContainer = styled.div({
  display: 'flex',
  flexDirection: 'column',
  flex: 1,
  justifyContent: 'center',
  padding: '14px',
});

const StyledModalAlert = styled.div({}, (props: { visible: boolean; closing: boolean }) => {
  let transform = '';
  if (props.visible && props.closing) {
    transform = 'scale(80%)';
  } else if (!props.visible) {
    transform = 'translateY(10px) scale(98%)';
  }

  return {
    display: 'flex',
    flexDirection: 'column',
    backgroundColor: colors.darkBlue,
    borderRadius: '11px',
    padding: '16px 0 16px 16px',
    maxHeight: '80vh',
    opacity: props.visible && !props.closing ? 1 : 0,
    transform,
    boxShadow: ' 0px 15px 35px 5px rgba(0,0,0,0.5)',
    transition: 'all 150ms ease-out',
  };
});

const StyledCustomScrollbars = styled(CustomScrollbars)({
  paddingRight: '16px',
});

const ModalAlertIcon = styled.div({
  display: 'flex',
  justifyContent: 'center',
  marginTop: '8px',
});

const ModalAlertButtonGroupContainer = styled.div({
  marginTop: measurements.buttonVerticalMargin,
});

const ModalAlertButtonContainer = styled.div({
  display: 'flex',
  flexDirection: 'column',
  marginRight: '16px',
});

interface IModalAlertProps {
  type?: ModalAlertType;
  iconColor?: string;
  message?: string;
  buttons: React.ReactNode[];
  children?: React.ReactNode;
  close?: () => void;
}

interface OpenState {
  isClosing: boolean;
  wasOpen: boolean;
}

export function ModalAlert(props: IModalAlertProps & { isOpen: boolean }) {
  const { isOpen, ...otherProps } = props;
  const activeModalContext = useContext(ActiveModalContext);
  const [openState, setOpenState] = useState<OpenState>({ isClosing: false, wasOpen: isOpen });

  const willExit = useWillExit();

  // Modal shouldn't prepare for being opened again while view is disappearing.
  const onTransitionEnd = useCallback(() => {
    if (!willExit) {
      setOpenState({ isClosing: false, wasOpen: isOpen });
    }
  }, [willExit, isOpen]);

  useEffect(() => {
    setOpenState(({ isClosing, wasOpen }) => ({
      isClosing: isClosing || (wasOpen && !isOpen),
      // Unmounting the Modal during view transitions result in a visual glitch.
      wasOpen: willExit ? wasOpen : isOpen,
    }));
  }, [isOpen]);

  if (!openState.wasOpen && !isOpen && !openState.isClosing) {
    return null;
  }

  return (
    <ModalAlertImpl
      {...activeModalContext}
      {...otherProps}
      closing={openState.isClosing}
      onTransitionEnd={onTransitionEnd}
    />
  );
}

interface IModalAlertState {
  visible: boolean;
}

interface IModalAlertImplProps extends IModalAlertProps, IModalContext {
  closing: boolean;
  onTransitionEnd: () => void;
}

class ModalAlertImpl extends React.Component<IModalAlertImplProps, IModalAlertState> {
  public state = { visible: false };

  private element = document.createElement('div');
  private modalRef = React.createRef<HTMLDivElement>();

  constructor(props: IModalAlertImplProps) {
    super(props);

    if (document.activeElement) {
      props.previousActiveElement.current = document.activeElement as HTMLElement;
    }
  }

  public componentDidMount() {
    this.props.setActiveModal(true);

    const modalContainer = document.getElementById(MODAL_CONTAINER_ID);
    if (modalContainer) {
      modalContainer.appendChild(this.element);
      this.modalRef.current?.focus();

      this.setState({ visible: true });
    } else {
      log.error('Modal container not found when mounting modal');
    }
  }

  public componentWillUnmount() {
    this.props.setActiveModal(false);

    const modalContainer = document.getElementById(MODAL_CONTAINER_ID);
    modalContainer?.removeChild(this.element);
  }

  public render() {
    return ReactDOM.createPortal(this.renderModal(), this.element);
  }

  private renderModal() {
    return (
      <BackAction action={this.close}>
        <ModalBackground visible={this.state.visible && !this.props.closing}>
          <ModalAlertContainer>
            <StyledModalAlert
              ref={this.modalRef}
              tabIndex={-1}
              role="dialog"
              aria-modal
              visible={this.state.visible}
              closing={this.props.closing}
              onTransitionEnd={this.onTransitionEnd}>
              <StyledCustomScrollbars>
                {this.props.type && (
                  <ModalAlertIcon>{this.renderTypeIcon(this.props.type)}</ModalAlertIcon>
                )}
                {this.props.message && <ModalMessage>{this.props.message}</ModalMessage>}
                {this.props.children}
              </StyledCustomScrollbars>

              <ModalAlertButtonGroupContainer>
                <AppButton.ButtonGroup>
                  {this.props.buttons.map((button, index) => (
                    <ModalAlertButtonContainer key={index}>{button}</ModalAlertButtonContainer>
                  ))}
                </AppButton.ButtonGroup>
              </ModalAlertButtonGroupContainer>
            </StyledModalAlert>
          </ModalAlertContainer>
        </ModalBackground>
      </BackAction>
    );
  }

  private close = () => {
    this.props.close?.();
  };

  private renderTypeIcon(type: ModalAlertType) {
    let source = '';
    let color = '';
    switch (type) {
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
    }
    return (
      <ImageView height={44} width={44} source={source} tintColor={this.props.iconColor ?? color} />
    );
  }

  private onTransitionEnd = (event: React.TransitionEvent<HTMLDivElement>) => {
    if (event.target === this.modalRef.current) {
      this.props.onTransitionEnd();
    }
  };
}

export const ModalMessage = styled.span(tinyText, {
  color: colors.white80,
  marginTop: '16px',
});
