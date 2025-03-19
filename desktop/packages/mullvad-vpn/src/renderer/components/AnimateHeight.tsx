import styled from 'styled-components';

const Container = styled.div<{ $expanded?: boolean }>`
  display: grid;
  grid-template-rows: ${({ $expanded }) => ($expanded ? '1fr' : '0fr')};
  transition: grid-template-rows 0.25s ease;
`;

const Content = styled.div`
  overflow: hidden;
`;

export interface AnimateHeightProps extends React.HTMLAttributes<HTMLDivElement> {
  expanded?: boolean;
  children?: React.ReactNode;
}

export const AnimateHeight = ({ expanded, children, ...props }: AnimateHeightProps) => {
  return (
    <Container $expanded={expanded} {...props}>
      <Content>{children}</Content>
    </Container>
  );
};
