import * as React from 'react';
import { Component } from 'reactxp';

interface IProps {
  children?: React.ReactNode;
  cacheKey?: number;
}

export default class AccordionContent extends Component<IProps> {
  public shouldComponentUpdate(nextProps: IProps) {
    return typeof this.props.cacheKey === 'number'
      ? this.props.cacheKey !== nextProps.cacheKey
      : true;
  }

  public render() {
    return <React.Fragment>{this.props.children}</React.Fragment>;
  }
}
