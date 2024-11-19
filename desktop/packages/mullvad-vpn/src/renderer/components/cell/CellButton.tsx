import React, { useContext } from 'react';
import styled from 'styled-components';

import { colors } from '../../../config.json';
import { spacings } from '../common-styles';
import { IImageViewProps } from '../ImageView';
import { CellDisabledContext } from './Container';
import { Icon } from './Label';
import { Row } from './Row';
import { CellSectionContext } from './Section';

interface IStyledCellButtonProps extends React.HTMLAttributes<HTMLButtonElement> {
  $selected?: boolean;
  $containedInSection: boolean;
}

const StyledCellButton = styled(Row)<IStyledCellButtonProps>((props) => {
  const backgroundColor = props.$selected
    ? colors.green
    : props.$containedInSection
      ? colors.blue40
      : colors.blue;
  const backgroundColorHover = props.$selected ? colors.green : colors.blue80;

  return {
    paddingRight: spacings.spacing5,
    paddingLeft: spacings.spacing5,
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

const StyledNavigationButtonIcon = styled('div')({
  display: 'grid',
  placeItems: 'center',
  width: '24px',
  height: '24px',
});

interface ICellNavigationButtonProps extends ICellButtonProps {
  isAriaDescription?: boolean;
  icon?: IImageViewProps;
}

export function CellNavigationButton({
  children,
  icon = {
    source: 'icon-chevron',
  },
  ...props
}: ICellNavigationButtonProps) {
  return (
    <CellButton {...props}>
      {children}
      <StyledNavigationButtonIcon>
        <Icon {...icon} />
      </StyledNavigationButtonIcon>
    </CellButton>
  );
}
