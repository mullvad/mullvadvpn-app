import { IconButton, IconButtonProps } from '../../../icon-button';

export type SectionTitleIconButtonProps = IconButtonProps;

function SectionTitleIconButton(props: SectionTitleIconButtonProps) {
  return <IconButton variant="secondary" {...props} />;
}

const SectionTitleIconButtonNamespace = Object.assign(SectionTitleIconButton, {
  Icon: IconButton.Icon,
});

export { SectionTitleIconButtonNamespace as SectionTitleIconButton };
