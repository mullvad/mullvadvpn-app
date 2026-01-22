import styled from 'styled-components';

export type AccordionContainerProps = React.ComponentPropsWithRef<'div'>;

export const StyledAccordionContainer = styled.div``;

export function AccordionContainer({ children, ...props }: AccordionContainerProps) {
  return <StyledAccordionContainer {...props}>{children}</StyledAccordionContainer>;
}
