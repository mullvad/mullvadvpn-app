import React from 'react';
import styled from 'styled-components';

import { colors } from '../../config.json';

interface IProps {
  id?: string;
  'aria-labelledby'?: string;
  'aria-describedby'?: string;
  isOn: boolean;
  onChange?: (isOn: boolean) => void;
  className?: string;
  disabled?: boolean;
  innerRef?: React.Ref<HTMLDivElement>;
}

const SwitchContainer = styled.div<{ disabled: boolean }>((props) => ({
  position: 'relative',
  width: '48px',
  height: '30px',
  borderColor: props.disabled ? colors.white20 : colors.white80,
  borderWidth: '2px',
  borderStyle: 'solid',
  borderRadius: '16px',
  padding: '2px',
}));

const Knob = styled.div<{ $isOn: boolean; disabled: boolean }>((props) => {
  let backgroundColor = props.$isOn ? colors.green : colors.red;
  if (props.disabled) {
    backgroundColor = props.$isOn ? colors.green40 : colors.red40;
  }

  return {
    position: 'absolute',
    height: '22px',
    borderRadius: '11px',
    transition: 'all 200ms linear',
    width: '22px',
    backgroundColor,
    // When enabled the button should be placed all the way to the right (100%) minus padding (2px)
    // minus it's own width (22px).
    left: props.$isOn ? 'calc(100% - 2px - 22px)' : '2px',
  };
});

export default class Switch extends React.PureComponent<IProps> {
  public render() {
    return (
      <SwitchContainer
        ref={this.props.innerRef}
        id={this.props.id}
        role="checkbox"
        aria-labelledby={this.props['aria-labelledby']}
        aria-describedby={this.props['aria-describedby']}
        aria-checked={this.props.isOn}
        onClick={this.handleClick}
        disabled={this.props.disabled ?? false}
        aria-disabled={this.props.disabled ?? false}
        tabIndex={-1}
        className={this.props.className}>
        <Knob disabled={this.props.disabled ?? false} $isOn={this.props.isOn} />
      </SwitchContainer>
    );
  }

  private handleClick = () => {
    if (!this.props.disabled) {
      this.props.onChange?.(!this.props.isOn);
    }
  };
}
