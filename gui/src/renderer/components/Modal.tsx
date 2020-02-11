import * as React from 'react';
import { Component, Styles, Text, View } from 'reactxp';
import { colors } from '../../config.json';
import ImageView from './ImageView';

const styles = {
  dialogBackground: Styles.createViewStyle({
    flex: 1,
    justifyContent: 'center',
    paddingLeft: 14,
    paddingRight: 14,
  }),
  dialog: Styles.createViewStyle({
    backgroundColor: colors.darkBlue,
    borderRadius: 11,
    padding: 16,
  }),
  dialogIcon: Styles.createViewStyle({
    alignItems: 'center',
    marginBottom: 12,
    marginTop: 4,
  }),
  dialogWarning: Styles.createTextStyle({
    fontFamily: 'Open Sans',
    fontSize: 16,
    fontWeight: '500',
    lineHeight: 20,
    color: colors.white80,
  }),
  dialogButtonContainer: Styles.createViewStyle({
    marginTop: 16,
  }),
};

export class ModalContent extends React.Component {
  public render() {
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
        {this.props.children}
      </div>
    );
  }
}

export class ModalAlert extends React.Component {
  public render() {
    return (
      <div
        style={{
          backgroundColor: 'rgba(0,0,0,0.7)',
          position: 'absolute',
          display: 'flex',
          flexDirection: 'column',
          flex: 1,
          top: 0,
          left: 0,
          right: 0,
          bottom: 0,
        }}>
        {this.props.children}
      </div>
    );
  }
}

interface IModalContainerProps {
  children?: React.ReactNode;
}

export class ModalContainer extends React.Component<IModalContainerProps> {
  public render() {
    return <div style={{ position: 'relative', flex: 1 }}>{this.props.children}</div>;
  }
}

export enum DialogType {
  Info = 1,
  Warning,
}

interface IDialogProps {
  type?: DialogType;
  message: string;
  buttons: React.ReactNode[];
}

export class Dialog extends Component<IDialogProps> {
  public render() {
    return (
      <View style={styles.dialogBackground}>
        <View style={styles.dialog}>
          {this.props.type && (
            <View style={styles.dialogIcon}>{this.renderTypeIcon(this.props.type)}</View>
          )}
          <Text style={styles.dialogWarning}>{this.props.message}</Text>
          {this.props.buttons.map((button, index) => (
            <View key={index} style={styles.dialogButtonContainer}>
              {button}
            </View>
          ))}
        </View>
      </View>
    );
  }

  private renderTypeIcon(type: DialogType) {
    let source = '';
    let color = '';
    switch (type) {
      case DialogType.Info:
        source = 'icon-alert';
        color = colors.white;
        break;
      case DialogType.Warning:
        source = 'icon-alert';
        color = colors.red;
        break;
    }
    return <ImageView height={44} width={44} source={source} tintColor={color} />;
  }
}
