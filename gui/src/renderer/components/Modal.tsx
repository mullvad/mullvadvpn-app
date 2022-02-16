import React, { useContext, useEffect, useMemo, useRef, useState } from 'react';
import ReactDOM from 'react-dom';
import styled from 'styled-components';
import { colors } from '../../config.json';
import log from '../../shared/logging';
import CustomScrollbars from './CustomScrollbars';
import { tinyText } from './common-styles';
import ImageView from './ImageView';

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
  transition: 'all 150ms ease-out',
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
  padding: '26px 14px 14px',
});

const StyledModalAlert = styled.div({}, (props: { visible: boolean }) => ({
  display: 'flex',
  flexDirection: 'column',
  backgroundColor: colors.darkBlue,
  borderRadius: '11px',
  padding: '16px 0 16px 16px',
  maxHeight: '80vh',
  opacity: props.visible ? 1 : 0,
  transform: props.visible ? '' : 'translateY(10px) scale(98%)',
  boxShadow: ' 0px 15px 35px 5px rgba(0,0,0,0.5)',
  transition: 'all 150ms ease-out',
}));

const StyledCustomScrollbars = styled(CustomScrollbars)({
  paddingRight: '16px',
});

const ModalAlertIcon = styled.div({
  display: 'flex',
  justifyContent: 'center',
  marginTop: '8px',
});

const ModalAlertButtonContainer = styled.div({
  display: 'flex',
  flexDirection: 'column',
  marginTop: '18px',
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

export function ModalAlert(props: IModalAlertProps) {
  const activeModalContext = useContext(ActiveModalContext);
  return <ModalAlertWithContext {...activeModalContext} {...props} />;
}

interface IModalAlertState {
  visible: boolean;
}

class ModalAlertWithContext extends React.Component<
  IModalAlertProps & IModalContext,
  IModalAlertState
> {
  public state = { visible: false };

  private element = document.createElement('div');
  private modalRef = React.createRef<HTMLDivElement>();

  constructor(props: IModalAlertProps & IModalContext) {
    super(props);

    if (document.activeElement) {
      props.previousActiveElement.current = document.activeElement as HTMLElement;
    }
  }

  public componentDidMount() {
    this.props.setActiveModal(true);
    // The `true` argument specifies that the event should be dispatched in the capture phase. This
    // makes sure that this component catches the event before the escape hatch.
    document.addEventListener('keydown', this.handleKeyPress, true);

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
    document.removeEventListener('keydown', this.handleKeyPress, true);

    const modalContainer = document.getElementById(MODAL_CONTAINER_ID);
    modalContainer?.removeChild(this.element);
  }

  public render() {
    return ReactDOM.createPortal(this.renderModal(), this.element);
  }

  private renderModal() {
    return (
      <ModalBackground visible={this.state.visible}>
        <ModalAlertContainer>
          <StyledModalAlert
            ref={this.modalRef}
            tabIndex={-1}
            role="dialog"
            aria-modal
            visible={this.state.visible}>
            <StyledCustomScrollbars>
              {this.props.type && (
                <ModalAlertIcon>{this.renderTypeIcon(this.props.type)}</ModalAlertIcon>
              )}
              {this.props.message && <ModalMessage>{this.props.message}</ModalMessage>}
              {this.props.children}
            </StyledCustomScrollbars>

            {this.props.buttons.map((button, index) => (
              <ModalAlertButtonContainer key={index}>{button}</ModalAlertButtonContainer>
            ))}
          </StyledModalAlert>
        </ModalAlertContainer>
      </ModalBackground>
    );
  }

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

  private handleKeyPress = (event: KeyboardEvent) => {
    if (event.key === 'Escape') {
      event.stopPropagation();
      this.props.close?.();
    }
  };
}

export const ModalMessage = styled.span(tinyText, {
  color: colors.white80,
  marginTop: '16px',
});
