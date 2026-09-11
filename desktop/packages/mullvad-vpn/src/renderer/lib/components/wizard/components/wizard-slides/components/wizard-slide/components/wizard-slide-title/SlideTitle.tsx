import styled from 'styled-components';

import { TitleLarge, type TitleLargeProps } from '../../../../../../../text';

export type WizardSlideTitleProps = TitleLargeProps;

export const StyledSlideTitle = styled(TitleLarge)`
  align-self: center;
  text-align: center;
`;

export function WizardSlideTitle(props: WizardSlideTitleProps) {
  return <StyledSlideTitle {...props}></StyledSlideTitle>;
}
