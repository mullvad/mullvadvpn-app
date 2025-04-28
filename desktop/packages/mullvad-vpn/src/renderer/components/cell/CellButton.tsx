import React, { useContext } from 'react';
import styled from 'styled-components';

import { Box, IconProps } from '../../lib/components';
import { DeprecatedColors, spacings } from '../../lib/foundations';
import { CellDisabledContext } from './Container';
import { CellTintedIcon } from './Label';
import { Row } from './Row';
import { CellSectionContext } from './Section';

interface IStyledCellButtonProps extends React.HTMLAttributes<HTMLButtonElement> {
  $selected?: boolean;
  $containedInSection: boolean;
}

const StyledCellButton = styled(Row)<IStyledCellButtonProps>((props) => {
  const backgroundColor = props.$selected
    ? DeprecatedColors.green
    : props.$containedInSection
      ? DeprecatedColors.blue40
      : DeprecatedColors.blue;
  const backgroundColorHover = props.$selected ? DeprecatedColors.green : DeprecatedColors.blue80;

  return {
    paddingRight: spacings.medium,
    paddingLeft: spacings.medium,
    flex: 1,
    alignContent: 'center',
    cursor: 'default',
    border: 'none',
    backgroundColor,
    '&&:not(:disabled):hover': {
      backgroundColor: props.onClick ? backgroundColorHover : backgroundColor,
    },
  };
});

interface ICellButtonProps extends React.ButtonHTMLAttributes<HTMLButtonElement> {
  selected?: boolean;
}

export const CellButton = styled(
  React.forwardRef(function Button(props: ICellButtonProps, ref: React.Ref<HTMLButtonElement>) {
    const { selected, ...otherProps } = props;
    const containedInSection = useContext(CellSectionContext);
    return (
      <CellDisabledContext.Provider value={props.disabled ?? false}>
        <StyledCellButton
          as="button"
          ref={ref}
          $selected={selected}
          $containedInSection={containedInSection}
          {...otherProps}
        />
      </CellDisabledContext.Provider>
    );
  }),
)({});

interface ICellNavigationButtonProps extends ICellButtonProps {
  isAriaDescription?: boolean;
  icon?: IconProps;
}

export function CellNavigationButton({
  children,
  icon = {
    icon: 'chevron-right',
  },
  ...props
}: ICellNavigationButtonProps) {
  return (
    <CellButton {...props}>
      {children}
      <Box $height="24px" $width="24px" center>
        <CellTintedIcon {...icon} />
      </Box>
    </CellButton>
  );
}
