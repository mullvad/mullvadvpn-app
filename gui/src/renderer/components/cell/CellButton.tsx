import React, { useContext } from 'react';
import styled from 'styled-components';

import { colors } from '../../../config.json';
import { CellDisabledContext } from './Container';
import { Icon } from './Label';
import { CellSectionContext } from './Section';

interface IStyledCellButtonProps extends React.HTMLAttributes<HTMLButtonElement> {
  selected?: boolean;
  containedInSection: boolean;
}

const StyledCellButton = styled.button({}, (props: IStyledCellButtonProps) => {
  const backgroundColor = props.selected
    ? colors.green
    : props.containedInSection
    ? colors.blue40
    : colors.blue;
  const backgroundColorHover = props.selected ? colors.green : colors.blue80;

  return {
    display: 'flex',
    minHeight: '44px',
    padding: '0 16px 0 22px',
    marginBottom: '1px',
    flex: 1,
    alignItems: 'center',
    alignContent: 'center',
    cursor: 'default',
    border: 'none',
    backgroundColor,
    ':not(:disabled):hover': {
      backgroundColor: props.onClick ? backgroundColorHover : backgroundColor,
    },
  };
});

interface ICellButtonProps extends React.ButtonHTMLAttributes<HTMLButtonElement> {
  selected?: boolean;
}

export const CellButton = styled(
  React.forwardRef(function Button(props: ICellButtonProps, ref: React.Ref<HTMLButtonElement>) {
    const containedInSection = useContext(CellSectionContext);
    return (
      <CellDisabledContext.Provider value={props.disabled ?? false}>
        <StyledCellButton ref={ref} containedInSection={containedInSection} {...props} />
      </CellDisabledContext.Provider>
    );
  }),
)({});

export function CellNavigationButton(props: ICellButtonProps) {
  const { children, ...otherProps } = props;

  return (
    <CellButton {...otherProps}>
      {children}
      <Icon height={12} width={7} source="icon-chevron" />
    </CellButton>
  );
}
