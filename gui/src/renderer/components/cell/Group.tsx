import React from 'react';
import styled from 'styled-components';

interface IStyledGroupProps {
  noMarginBottom?: boolean;
}

const StyledGroup = styled.div({}, (props: IStyledGroupProps) => ({
  display: 'flex',
  flexDirection: 'column',
  flex: 1,
  marginBottom: props.noMarginBottom ? '0px' : '20px',
}));

const StyledCellWrapper = styled.div({
  display: 'flex',
  flexDirection: 'column',
  flex: 1,
  marginBottom: '1px',
});

interface IGroupProps extends IStyledGroupProps {
  children: React.ReactNode | React.ReactNode[];
}

export function Group(props: IGroupProps) {
  const children = React.Children.toArray(props.children);
  return (
    <StyledGroup noMarginBottom={props.noMarginBottom}>
      {children.map((child, index) =>
        index === children.length - 1 ? (
          child
        ) : (
          <StyledCellWrapper key={index}>{child}</StyledCellWrapper>
        ),
      )}
    </StyledGroup>
  );
}
