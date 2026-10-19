import { Text, type TextProps } from '../../../text';

export type NoteTextProps = TextProps;

export function NoteText(props: NoteTextProps) {
  return <Text color="whiteAlpha60" variant="labelTinySemiBold" {...props} />;
}
