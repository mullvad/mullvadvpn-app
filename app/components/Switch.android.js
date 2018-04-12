// @flow
import * as React from 'react';
import { Switch as _Switch } from 'react-native';

export type SwitchProps = {
  isOn: boolean;
  onChange?: (isOn: boolean) => void;
};

type State = {
};

export default class Switch extends React.Component<SwitchProps, State> {
  static defaultProps: SwitchProps = {
    isOn: false,
    onChange: ()=>{},
  };

  state = {
  };

  render() {
    const { isOn, ...otherProps } = this.props;
    return (
      <_Switch { ...otherProps }
        value={ isOn }
        onValueChange={ this.props.onChange(isOn) } />
    );
  }
}
