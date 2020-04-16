import * as React from 'react';
import ReactDOM from 'react-dom';
import { Component, Styles, Text, View } from 'reactxp';
import { colors } from '../../config.json';
import { Scheduler } from '../../shared/scheduler';
import ImageView from './ImageView';

const MODAL_CONTAINER_ID = 'modalContainer';

const styles = {
  modalAlertBackground: Styles.createViewStyle({
    flex: 1,
    justifyContent: 'center',
    paddingHorizontal: 14,
    paddingTop: 26,
    paddingBottom: 14,
  }),
  modalAlert: Styles.createViewStyle({
    backgroundColor: colors.darkBlue,
    borderRadius: 11,
    padding: 16,
  }),
  modalAlertIcon: Styles.createViewStyle({
    alignItems: 'center',
    marginTop: 8,
  }),
  modalAlertMessageContainer: Styles.createViewStyle({
    // marginTop: 16,
  }),
  modalAlertMessage: Styles.createTextStyle({
    fontFamily: 'Open Sans',
    fontSize: 14,
    fontWeight: '500',
    lineHeight: 20,
    color: colors.white80,
    marginTop: 16,
  }),
  modalAlertButtonContainer: Styles.createViewStyle({
    marginTop: 16,
  }),
};

interface IModalContentProps {
  children?: React.ReactNode;
}

export const ModalContent: React.FC = (props: IModalContentProps) => {
  return (
    <div
      style={{
        position: 'absolute',
        display: 'flex',
        flexDirection: 'column',
        flex: 1,
        top: 0,
        left: 0,
        right: 0,
        bottom: 0,
      }}>
      {props.children}
    </div>
  );
};

interface IModalBackgroundProps {
  children?: React.ReactNode;
}

const ModalBackground: React.FC = (props: IModalBackgroundProps) => {
  return (
    <div
      style={{
        backgroundColor: 'rgba(0,0,0,0.5)',
        position: 'absolute',
        display: 'flex',
        flexDirection: 'column',
        flex: 1,
        top: 0,
        left: 0,
        right: 0,
        bottom: 0,
      }}>
      {props.children}
    </div>
  );
};

interface IModalContainerProps {
  children?: React.ReactNode;
}

export const ModalContainer: React.FC = (props: IModalContainerProps) => {
  return (
    <div id={MODAL_CONTAINER_ID} style={{ position: 'relative', flex: 1 }}>
      <ModalContent>{props.children}</ModalContent>
    </div>
  );
};

export enum ModalAlertType {
  Info = 1,
  Warning,
}

interface IModalAlertProps {
  type?: ModalAlertType;
  message?: string;
  buttons: React.ReactNode[];
  children?: React.ReactNode;
}

export class ModalAlert extends Component<IModalAlertProps> {
  private element = document.createElement('div');
  private modalContainer?: Element;
  private appendScheduler = new Scheduler();

  public componentDidMount() {
    const modalContainer = document.getElementById(MODAL_CONTAINER_ID);
    if (modalContainer) {
      this.modalContainer = modalContainer;
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
        <View style={styles.modalAlertBackground}>
          <View style={styles.modalAlert}>
            {this.props.type && (
              <View style={styles.modalAlertIcon}>{this.renderTypeIcon(this.props.type)}</View>
            )}
            <View style={styles.modalAlertMessageContainer}>
              {this.props.message && <ModalMessage>{this.props.message}</ModalMessage>}
              {this.props.children}
            </View>
            {this.props.buttons.map((button, index) => (
              <View key={index} style={styles.modalAlertButtonContainer}>
                {button}
              </View>
            ))}
          </View>
        </View>
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
    return <ImageView height={44} width={44} source={source} tintColor={color} />;
  }
}

interface IModalMessageProps {
  children?: string;
}

export function ModalMessage(props: IModalMessageProps) {
  return <Text style={styles.modalAlertMessage}>{props.children}</Text>;
}
