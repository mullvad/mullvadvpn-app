import * as React from 'react';
import ReactDOM from 'react-dom';
import styled from 'styled-components';
import { colors } from '../../config.json';
import { Scheduler } from '../../shared/scheduler';
import ImageView from './ImageView';

const MODAL_CONTAINER_ID = 'modalContainer';

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

const ModalBackground = styled.div({
  backgroundColor: 'rgba(0,0,0,0.5)',
  position: 'absolute',
  display: 'flex',
  flexDirection: 'column',
  flex: 1,
  top: 0,
  left: 0,
  right: 0,
  bottom: 0,
});

export const StyledModalContainer = styled.div({
  position: 'relative',
  flex: 1,
});

interface IModalContainerProps {
  children?: React.ReactNode;
}

export function ModalContainer(props: IModalContainerProps) {
  return (
    <StyledModalContainer id={MODAL_CONTAINER_ID}>
      <ModalContent>{props.children}</ModalContent>
    </StyledModalContainer>
  );
}

export enum ModalAlertType {
  Info = 1,
  Warning,
}

const ModalAlertContainer = styled.div({
  display: 'flex',
  flexDirection: 'column',
  flex: 1,
  justifyContent: 'center',
  padding: '26px 14px 14px',
});

const StyledModalAlert = styled.div({
  display: 'flex',
  flexDirection: 'column',
  backgroundColor: colors.darkBlue,
  borderRadius: '11px',
  padding: '16px',
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
});

interface IModalAlertProps {
  type?: ModalAlertType;
  iconColor?: string;
  message?: string;
  buttons: React.ReactNode[];
  children?: React.ReactNode;
}

export class ModalAlert extends React.Component<IModalAlertProps> {
  private element = document.createElement('div');
  private modalContainer?: Element;
  private appendScheduler = new Scheduler();

  public componentDidMount() {
    const modalContainer = document.getElementById(MODAL_CONTAINER_ID);
    if (modalContainer) {
      this.modalContainer = modalContainer;

      // Mounting the container element immediately results in a graphical issue with the dialog
      // first rendering with the wrong proportions and then changing to the correct proportions.
      // Postponing it to the next event cycle solves this issue.
      this.appendScheduler.schedule(() => {
        modalContainer.appendChild(this.element);
      });
    } else {
      throw Error('Modal container not found when mounting modal');
    }
  }

  public componentWillUnmount() {
    this.appendScheduler.cancel();

    if (this.modalContainer) {
      this.modalContainer.removeChild(this.element);
    }
  }

  public render() {
    return ReactDOM.createPortal(this.renderModal(), this.element);
  }

  private renderModal() {
    return (
      <ModalBackground>
        <ModalAlertContainer>
          <StyledModalAlert>
            {this.props.type && (
              <ModalAlertIcon>{this.renderTypeIcon(this.props.type)}</ModalAlertIcon>
            )}
            {this.props.message && <ModalMessage>{this.props.message}</ModalMessage>}
            {this.props.children}
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
      case ModalAlertType.Info:
        source = 'icon-alert';
        color = colors.white;
        break;
      case ModalAlertType.Warning:
        source = 'icon-alert';
        color = colors.red;
        break;
    }
    return (
      <ImageView height={44} width={44} source={source} tintColor={this.props.iconColor ?? color} />
    );
  }
}

export const ModalMessage = styled.span({
  fontFamily: 'Open Sans',
  fontSize: '13px',
  fontWeight: 500,
  lineHeight: '20px',
  color: colors.white80,
  marginTop: '16px',
});
