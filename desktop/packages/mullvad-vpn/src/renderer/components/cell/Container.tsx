import React from 'react';
import styled from 'styled-components';

import { spacings } from '../../tokens';
import { Row } from './Row';

const StyledContainer = styled(Row)({
  padding: `0 ${spacings.spacing5}`,
});

export const CellDisabledContext = React.createContext<boolean>(false);

interface IContainerProps extends React.HTMLAttributes<HTMLDivElement> {
  disabled?: boolean;
}

export const Container = React.forwardRef(function ContainerT(
  props: IContainerProps,
  ref: React.Ref<HTMLDivElement>,
) {
  const { disabled, ...otherProps } = props;
  return (
    <CellDisabledContext.Provider value={disabled ?? false}>
      <StyledContainer ref={ref} {...otherProps} />
    </CellDisabledContext.Provider>
  );
});
