import * as React from 'react';
import { Component, Styles, Text, Types } from 'reactxp';
import { colors } from '../../config.json';
import { messages } from '../../shared/gettext';

export enum SecuredDisplayStyle {
  secured,
  blocked,
  securing,
  unsecured,
  failedToSecure,
}

interface IProps {
  displayStyle: SecuredDisplayStyle;
  style: Types.TextStyleRuleSet;
}

const styles = {
  securing: Styles.createTextStyle({
    color: colors.white,
  }),
  secured: Styles.createTextStyle({
    color: colors.green,
  }),
  unsecured: Styles.createTextStyle({
    color: colors.red,
  }),
};

export default class SecuredLabel extends Component<IProps> {
  public render() {
    return <Text style={[this.props.style, this.getTextStyle()]}>{this.getText()}</Text>;
  }

  private getText() {
    switch (this.props.displayStyle) {
      case SecuredDisplayStyle.secured:
        return messages.gettext('SECURE CONNECTION');

      case SecuredDisplayStyle.blocked:
        return messages.gettext('BLOCKED CONNECTION');

      case SecuredDisplayStyle.securing:
        return messages.gettext('CREATING SECURE CONNECTION');

      case SecuredDisplayStyle.unsecured:
        return messages.gettext('UNSECURED CONNECTION');

      case SecuredDisplayStyle.failedToSecure:
        return messages.gettext('FAILED TO SECURE CONNECTION');
    }
  }

  private getTextStyle() {
    switch (this.props.displayStyle) {
      case SecuredDisplayStyle.secured:
      case SecuredDisplayStyle.blocked:
        return styles.secured;

      case SecuredDisplayStyle.securing:
        return styles.securing;

      case SecuredDisplayStyle.unsecured:
      case SecuredDisplayStyle.failedToSecure:
        return styles.unsecured;
    }
  }
}
