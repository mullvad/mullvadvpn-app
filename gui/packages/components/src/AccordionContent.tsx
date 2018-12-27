import * as React from 'react';
import { Component } from 'reactxp';

interface IProps {
  children?: React.ReactNode;
  cacheKey: number;
}

export default class AccordionContent extends Component<IProps> {
  public shouldComponentUpdate(nextProps: IProps) {
    return this.props.cacheKey !== nextProps.cacheKey;
  }

  public render() {
    return <React.Fragment>{this.props.children}</React.Fragment>;
  }
}
