import styled from 'styled-components';

export interface ListItemItemProps {
  children: React.ReactNode;
}

const StyledDiv = styled.div`
  min-height: 44px;
  width: 100%;
  display: grid;
  grid-template-columns: 1fr;
  &&:has(> :last-child:nth-child(2)) {
    grid-template-columns: 1fr 44px;
  }
`;

export const ListItemItem = ({ children }: ListItemItemProps) => {
  return <StyledDiv>{children}</StyledDiv>;
};
