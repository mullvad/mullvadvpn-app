import React, { useCallback, useContext, useEffect, useMemo, useRef, useState } from 'react';
import ReactDOM from 'react-dom';
import styled from 'styled-components';

import log from '../../shared/logging';
import { Icon, IconProps, Spinner } from '../lib/components';
import { Colors } from '../lib/foundations';
import { IconBadge } from '../lib/icon-badge';
import { useEffectEvent } from '../lib/utility-hooks';
import * as AppButton from './AppButton';
import { measurements, normalText, tinyText } from './common-styles';
import CustomScrollbars from './CustomScrollbars';
import { BackAction } from './KeyboardNavigation';
import { SmallButtonGrid } from './SmallButton';

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

const ModalBackground = styled.div<{ $visible: boolean }>((props) => ({
  backgroundColor: props.$visible ? 'rgba(0,0,0,0.5)' : 'rgba(0,0,0,0)',
  backdropFilter: props.$visible ? 'blur(1.5px)' : '',
  position: 'absolute',
  display: 'flex',
  flexDirection: 'column',
  flex: 1,
  top: 0,
  left: 0,
  right: 0,
  bottom: 0,
  transition: 'background-color 150ms ease-out',
  pointerEvents: props.$visible ? 'auto' : 'none',
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

  loading,
  success,
  failure,
}

const ModalAlertContainer = styled.div({
  display: 'flex',
  flexDirection: 'column',
  flex: 1,
  justifyContent: 'center',
  padding: '14px',
});

const StyledModalAlert = styled.div<{ $visible: boolean; $closing: boolean }>((props) => {
  let transform = '';
  if (props.$visible && props.$closing) {
    transform = 'scale(80%)';
  } else if (!props.$visible) {
    transform = 'translateY(10px) scale(98%)';
  }

  return {
    display: 'flex',
    flexDirection: 'column',
    backgroundColor: Colors.darkBlue,
    borderRadius: '11px',
    padding: '16px 0 16px 16px',
    maxHeight: '80vh',
    opacity: props.$visible && !props.$closing ? 1 : 0,
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

const StyledSmallButtonGrid = styled(SmallButtonGrid)({
  marginRight: '16px',
});

const ModalAlertButtonContainer = styled.div({
  display: 'flex',
  flexDirection: 'column',
  marginRight: '16px',
});

interface IModalAlertProps {
  type?: ModalAlertType;
  iconColor?: string;
  title?: string;
  message?: string | Array<string>;
  buttons?: React.ReactNode[];
  gridButtons?: React.ReactNode[];
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

  // Modal shouldn't prepare for being opened again while view is disappearing.
  const onTransitionEnd = useCallback(() => {
    setOpenState({ isClosing: false, wasOpen: isOpen });
  }, [isOpen]);

  const onOpenStateChange = useEffectEvent((isOpen: boolean) => {
    setOpenState(({ isClosing, wasOpen }) => ({
      isClosing: isClosing || (wasOpen && !isOpen),
      // Unmounting the Modal during view transitions result in a visual glitch.
      wasOpen: isOpen,
    }));
  });

  useEffect(() => onOpenStateChange(isOpen), [isOpen]);

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
    const messages =
      typeof this.props.message === 'string' ? [this.props.message] : this.props.message;

    return (
      <BackAction action={this.close}>
        <ModalBackground $visible={this.state.visible && !this.props.closing}>
          <ModalAlertContainer>
            <StyledModalAlert
              ref={this.modalRef}
              tabIndex={-1}
              role="dialog"
              aria-modal
              $visible={this.state.visible}
              $closing={this.props.closing}
              onTransitionEnd={this.onTransitionEnd}>
              <StyledCustomScrollbars>
                {this.props.type && (
                  <ModalAlertIcon>{this.renderTypeIcon(this.props.type)}</ModalAlertIcon>
                )}
                {this.props.title && <ModalTitle>{this.props.title}</ModalTitle>}
                {messages &&
                  messages.map((message) => <ModalMessage key={message}>{message}</ModalMessage>)}
                {this.props.children}
              </StyledCustomScrollbars>

              <ModalAlertButtonGroupContainer>
                {this.props.gridButtons && (
                  <StyledSmallButtonGrid>{this.props.gridButtons}</StyledSmallButtonGrid>
                )}
                {this.props.buttons && (
                  <AppButton.ButtonGroup>
                    {this.props.buttons.map((button, index) => (
                      <ModalAlertButtonContainer key={index}>{button}</ModalAlertButtonContainer>
                    ))}
                  </AppButton.ButtonGroup>
                )}
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
    let source: IconProps['icon'] | undefined = undefined;
    let color = undefined;
    switch (type) {
      case ModalAlertType.info:
        source = 'info-circle';
        color = Colors.white;
        break;
      case ModalAlertType.caution:
        source = 'alert-circle';
        color = Colors.white;
        break;
      case ModalAlertType.warning:
        source = 'alert-circle';
        color = Colors.red;
        break;
      case ModalAlertType.loading:
        return <Spinner size="big" />;
      case ModalAlertType.success:
        return <IconBadge state="positive" />;
      case ModalAlertType.failure:
        return <IconBadge state="negative" />;
    }

    return <Icon size="big" icon={source} color={color} />;
  }

  private onTransitionEnd = (event: React.TransitionEvent<HTMLDivElement>) => {
    if (event.target === this.modalRef.current) {
      this.props.onTransitionEnd();
    }
  };
}

const ModalTitle = styled.h1(normalText, {
  color: Colors.white,
  fontWeight: 600,
  margin: '18px 0 0 0',
});

export const ModalMessage = styled.span(tinyText, {
  color: Colors.white80,
  marginTop: '16px',

  [`${ModalTitle} ~ &&`]: {
    marginTop: '6px',
  },
});

export const ModalMessageList = styled.ul({
  listStyle: 'disc outside',
  paddingLeft: '20px',
  color: Colors.white80,
});
