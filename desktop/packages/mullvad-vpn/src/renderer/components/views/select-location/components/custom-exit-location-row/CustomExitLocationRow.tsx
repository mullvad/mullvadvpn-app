import {
  getButtonColor,
  StyledLocationRowButton,
  StyledLocationRowContainerWithMargin,
  StyledLocationRowLabel,
} from '../any-location-list-item/LocationRowStyles';
import type { SpecialLocationRowProps } from '../special-location-row';

export interface SpecialLocationRowInnerProps<T>
  extends Omit<SpecialLocationRowProps<T>, 'onSelect'> {
  onSelect: () => void;
}

export function CustomExitLocationRow(props: SpecialLocationRowInnerProps<undefined>) {
  const selectedRef = props.source.selected ? props.selectedElementRef : undefined;
  const background = getButtonColor(props.source.selected, 0, props.source.disabled);
  return (
    <StyledLocationRowContainerWithMargin ref={selectedRef}>
      <StyledLocationRowButton $level={0} {...background}>
        <StyledLocationRowLabel>{props.source.label}</StyledLocationRowLabel>
      </StyledLocationRowButton>
    </StyledLocationRowContainerWithMargin>
  );
}
