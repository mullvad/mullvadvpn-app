import { Divider, type DividerProps } from '../../../divider';

export type SectionTitleDividerProps = DividerProps;

export function SectionTitleDivider(props: SectionTitleDividerProps) {
  return <Divider decorative {...props} />;
}
