import * as React from 'react';
import { Component, Styles, Text, View } from 'reactxp';
import { colors } from '../../config.json';

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

export const ModalContent: React.FC = ({ children }) => {
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
      {children}
    </div>
  );
};

const ModalBackground: React.FC = ({ children }) => {
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
      {children}
    </div>
  );
};

export const ModalContainer: React.FC = ({ children }) => {
  return <div style={{ position: 'relative', flex: 1 }}>{children}</div>;
};

interface IModalAlertProps {
  message: string;
  buttons: React.ReactNode[];
}

export class ModalAlert extends Component<IModalAlertProps> {
  public render() {
    return (
      <ModalBackground>
        <View style={styles.modalAlertBackground}>
          <View style={styles.modalAlert}>
            <Text style={styles.modalAlertMessage}>{this.props.message}</Text>
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
}
