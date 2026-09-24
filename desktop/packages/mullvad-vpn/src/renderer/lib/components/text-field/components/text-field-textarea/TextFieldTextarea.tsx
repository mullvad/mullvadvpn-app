import styled from 'styled-components';

export type TextFieldTextareaProps = React.PropsWithChildren;

export const StyledTextFieldTextarea = styled.div`
  position: relative;
  flex-grow: 1;
`;

function TextFieldTextarea({ children }: TextFieldTextareaProps) {
  return <StyledTextFieldTextarea>{children}</StyledTextFieldTextarea>;
}

const TextFieldTextAreaNamespace = Object.assign(TextFieldTextarea, {});

export { TextFieldTextAreaNamespace as TextFieldTextArea };
