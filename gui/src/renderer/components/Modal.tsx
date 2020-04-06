import * as React from 'react';
import ReactDOM from 'react-dom';
import { Component, Styles, Text, View } from 'reactxp';
import { colors } from '../../config.json';
import ImageView from './ImageView';

const MODAL_CONTAINER_ID = 'modalContainer';

const styles = {
  modalAlertBackground: Styles.createViewStyle({
    flex: 1,
    justifyContent: 'center',
    paddingLeft: 14,
    paddingRight: 14,
  }),
  modalAlert: Styles.createViewStyle({
    backgroundColor: colors.darkBlue,
    borderRadius: 11,
    padding: 16,
  }),
  modalAlertIcon: Styles.createViewStyle({
    alignItems: 'center',
    marginBottom: 12,
    marginTop: 4,
  }),
  modalAlertMessage: Styles.createTextStyle({
    fontFamily: 'Open Sans',
    fontSize: 16,
    fontWeight: '500',
    lineHeight: 20,
    color: colors.white80,
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
  public render() {
    const modalContainer = document.getElementById(MODAL_CONTAINER_ID);
    if (modalContainer !== null) {
      return ReactDOM.createPortal(this.renderModal(), modalContainer);
    } else {
      throw Error('Modal container not found when rendering modal');
    }
  }

  private renderModal() {
    return (
      <ModalBackground>
        <View style={styles.modalAlertBackground}>
          <View style={styles.modalAlert}>
            {this.props.type && (
              <View style={styles.modalAlertIcon}>{this.renderTypeIcon(this.props.type)}</View>
            )}
            {this.props.message && (
              <Text style={styles.modalAlertMessage}>{this.props.message}</Text>
            )}
            {this.props.children}
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
