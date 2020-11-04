import React, { useContext } from 'react';
import styled from 'styled-components';
import { colors } from '../../../config.json';
import { CellDisabledContext } from './Container';
import { CellSectionContext } from './Section';

interface IStyledCellButtonProps {
  selected?: boolean;
  containedInSection: boolean;
}

const StyledCellButton = styled.button({}, (props: IStyledCellButtonProps) => ({
  display: 'flex',
  padding: '0 16px 0 22px',
  marginBottom: '1px',
  flex: 1,
  alignItems: 'center',
  alignContent: 'center',
  cursor: 'default',
  border: 'none',
  backgroundColor: props.selected
    ? colors.green
    : props.containedInSection
    ? colors.blue40
    : colors.blue,
  ':not(:disabled):hover': {
    backgroundColor: props.selected ? colors.green : colors.blue80,
  },
}));

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
