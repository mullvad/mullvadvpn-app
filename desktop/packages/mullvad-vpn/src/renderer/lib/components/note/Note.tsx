import styled from 'styled-components';

import { spacings } from '../../foundations';
import { NoteText } from './components';
import { NoteIcon } from './components/note-icon/NoteIcon';

export type NoteProps = React.ComponentPropsWithRef<'div'>;

export const StyledNote = styled.div`
  display: flex;
  flex-direction: row;
  align-items: center;
  gap: ${spacings.small};
`;

function Note({ children, ...props }: NoteProps) {
  return <StyledNote {...props}>{children}</StyledNote>;
}

const NoteNamespace = Object.assign(Note, {
  Text: NoteText,
  Icon: NoteIcon,
});

export { NoteNamespace as Note };
